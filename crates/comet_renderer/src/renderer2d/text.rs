use super::shaders::FONT_SHADER;
use super::*;

#[derive(Clone, Copy)]
struct ResolvedGlyph {
    region: TextureRegion,
    representation: GlyphRepresentation,
    distance_range: f32,
}
const BITMAP_TEXT_THRESHOLD: f32 = 18.0;
const FONT_ATLAS_SIZE: u32 = 1024;

impl Renderer2D {
    fn ensure_font_atlas(&mut self) -> bool {
        if self
            .render_state
            .resources()
            .get_asset_atlas_handle("font_atlas")
            .is_some()
        {
            return true;
        }

        let mut atlas = comet_assets::TextureAtlas::with_capacity(FONT_ATLAS_SIZE);
        atlas.clear_atlas_image();
        let Some(atlas_handle) = self.asset_provider.add(atlas) else {
            error!("Failed to allocate font atlas asset");
            return false;
        };
        let font_texture = Arc::new(GpuTexture::create_2d_texture(
            self.render_state.device(),
            FONT_ATLAS_SIZE,
            FONT_ATLAS_SIZE,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            wgpu::FilterMode::Linear,
            Some("FontAtlas"),
        ));
        self.render_state
            .resources_mut()
            .insert_asset_atlas_handle("font_atlas".to_string(), atlas_handle);
        self.render_state
            .resources_mut()
            .insert_gpu_texture("font_atlas".to_string(), font_texture.clone());

        let format = self.render_state.config().format;
        let width = self.render_state.config().width;
        let height = self.render_state.config().height;
        self.graph.add_node(
            PassNode::new(
                "Font",
                FONT_SHADER,
                wgpu::PrimitiveTopology::TriangleList,
                Some(font_texture.clone()),
                vec!["Universal"],
                LoadOp::Load,
            ),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );
        self.graph.add_node(
            PassNode::new(
                "ScreenFont",
                FONT_SHADER,
                wgpu::PrimitiveTopology::TriangleList,
                Some(font_texture),
                vec!["Gizmo"],
                LoadOp::Load,
            ),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );
        true
    }

    fn font_variant(
        &self,
        font: comet_assets::Asset<comet_assets::Font>,
        requested_size: comet_math::Px,
    ) -> (FontKey, GlyphRepresentation, f32, f32) {
        let requested = requested_size.pixels().max(1.0);
        let settings = self
            .asset_provider
            .with(font, |font| font.settings())
            .unwrap_or_default();
        let (generation_size, representation) = match settings.rasterization() {
            comet_assets::FontRasterization::Auto if requested > BITMAP_TEXT_THRESHOLD => (
                settings.mtsdf_generation_size().pixels(),
                GlyphRepresentation::Mtsdf,
            ),
            comet_assets::FontRasterization::Mtsdf => (
                settings.mtsdf_generation_size().pixels(),
                GlyphRepresentation::Mtsdf,
            ),
            comet_assets::FontRasterization::Pixel => {
                (requested.round().max(1.0), GlyphRepresentation::Pixel)
            }
            _ => (requested.round().max(1.0), GlyphRepresentation::Bitmap),
        };
        let distance_range = if representation == GlyphRepresentation::Mtsdf {
            settings.mtsdf_range() as f32
        } else {
            0.0
        };
        (
            FontKey {
                index: font.index(),
                generation: font.generation(),
                size_bits: generation_size.to_bits(),
            },
            representation,
            requested / generation_size,
            distance_range,
        )
    }

    fn ensure_font_variant(
        &mut self,
        font: comet_assets::Asset<comet_assets::Font>,
        font_key: FontKey,
        representation: GlyphRepresentation,
    ) -> bool {
        if self.font_cache.contains_key(&font_key) {
            return true;
        }
        if !self.ensure_font_atlas() {
            return false;
        }

        let size = comet_math::px(f32::from_bits(font_key.size_bits));
        let Some(font_data) = self.asset_provider.with(font, |font| font.clone()) else {
            error!("Font handle {:?} is unavailable", font);
            return false;
        };
        let rasterized = match representation {
            GlyphRepresentation::Bitmap | GlyphRepresentation::Pixel => font_data.rasterize(size),
            GlyphRepresentation::Mtsdf => {
                font_data.rasterize_mtsdf(size, font_data.settings().mtsdf_range())
            }
        };
        let Some((mut glyphs, line_height)) = rasterized else {
            return false;
        };
        glyphs.sort_by_key(|glyph| {
            std::cmp::Reverse((
                glyph.render.width().max(glyph.render.height()),
                glyph.render.width() * glyph.render.height(),
            ))
        });
        let atlas_handle = self
            .render_state
            .resources()
            .get_asset_atlas_handle("font_atlas")
            .unwrap();

        for glyph in glyphs {
            let Some(character) = glyph.name.chars().next() else {
                continue;
            };
            let key = GlyphKey {
                font: font_key,
                character,
                representation,
            };
            let name = format!(
                "{}:{}:{}:{}:{}",
                font_key.index,
                font_key.generation,
                font_key.size_bits,
                match representation {
                    GlyphRepresentation::Bitmap => 0,
                    GlyphRepresentation::Mtsdf => 1,
                    GlyphRepresentation::Pixel => 2,
                },
                character as u32,
            );
            let width = glyph.render.width();
            let height = glyph.render.height();
            let insertion = self
                .asset_provider
                .with_mut(atlas_handle, |atlas| {
                    atlas.insert_named(
                        name,
                        width,
                        height,
                        2,
                        glyph.advance,
                        glyph.offset_x,
                        glyph.offset_y,
                    )
                })
                .flatten();
            let Some((blit_position, region)) = insertion else {
                error!("Font atlas is full while inserting a font variant");
                return false;
            };
            if let Some((x, y)) = blit_position {
                if let Some(texture) = self.render_state.resources().get_gpu_texture("font_atlas") {
                    texture.write_region(
                        self.render_state.queue(),
                        x,
                        y,
                        glyph.render.as_bytes(),
                        width,
                        height,
                    );
                }
            }
            self.glyph_cache.insert(key, region);
        }
        self.font_cache.insert(font_key, line_height);
        true
    }
    fn get_glyph(
        &self,
        character: char,
        font: FontKey,
        representation: GlyphRepresentation,
        distance_range: f32,
    ) -> ResolvedGlyph {
        let region = self
            .glyph_cache
            .get(&GlyphKey {
                font,
                character,
                representation,
            })
            .or_else(|| {
                self.glyph_cache.get(&GlyphKey {
                    font,
                    character: ' ',
                    representation,
                })
            })
            .copied()
            .unwrap_or_else(|| fatal!("No glyph or fallback for '{}' in font atlas", character));
        ResolvedGlyph {
            region,
            representation,
            distance_range,
        }
    }

    pub fn precompute_text_bounds(
        &mut self,
        text: &str,
        font: comet_assets::Asset<comet_assets::Font>,
        size: comet_math::ScreenUnit,
    ) -> v2 {
        let size = size.resolve(self.scale_factor() as f32);
        let mut bounds = v2::ZERO;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        self.add_text_to_buffers(
            text,
            font,
            size,
            1.0,
            v2::ZERO,
            wgpu::Color::WHITE,
            comet_ecs::Anchor::TopLeft,
            comet_ecs::TextJustification::Left,
            &mut bounds,
            &mut vertices,
            &mut indices,
        );
        bounds
    }

    pub fn add_text_to_buffers(
        &mut self,
        text: &str,
        font: comet_assets::Asset<comet_assets::Font>,
        raster_size: comet_math::Px,
        geometry_scale: f32,
        position: comet_math::v2,
        color: wgpu::Color,
        anchor: comet_ecs::Anchor,
        justification: comet_ecs::TextJustification,
        bounds: &mut comet_math::v2,
        vertex_data: &mut Vec<Vertex>,
        index_data: &mut Vec<u16>,
    ) {
        let (cache_key, representation, variant_scale, distance_range) =
            self.font_variant(font, raster_size);
        self.ensure_font_variant(font, cache_key, representation);
        let generation_size = f32::from_bits(cache_key.size_bits);
        let line_height_px = self
            .font_cache
            .get(&cache_key)
            .copied()
            .unwrap_or(generation_size);
        let glyph_scale = geometry_scale * variant_scale;

        let vert_color = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];

        let line_height = line_height_px * glyph_scale;
        let screen_position = position;

        let lines: Vec<Vec<ResolvedGlyph>> = text
            .split('\n')
            .map(|line| {
                line.chars()
                    .map(|character| {
                        let character = if character == '\t' { ' ' } else { character };
                        self.get_glyph(character, cache_key, representation, distance_range)
                    })
                    .collect()
            })
            .collect();

        let line_widths: Vec<f32> = lines
            .iter()
            .map(|line| line.iter().map(|glyph| glyph.region.advance()).sum::<f32>() * glyph_scale)
            .collect();
        let max_line_width = line_widths.iter().copied().fold(0.0, f32::max);
        let block_height = lines.len() as f32 * line_height;
        bounds.set_x(max_line_width);
        bounds.set_y(block_height);

        let (anchor_x, anchor_y) = match anchor {
            comet_ecs::Anchor::TopLeft => (0.0, 0.0),
            comet_ecs::Anchor::TopCenter => (0.5, 0.0),
            comet_ecs::Anchor::TopRight => (1.0, 0.0),
            comet_ecs::Anchor::CenterLeft => (0.0, 0.5),
            comet_ecs::Anchor::Center => (0.5, 0.5),
            comet_ecs::Anchor::CenterRight => (1.0, 0.5),
            comet_ecs::Anchor::BottomLeft => (0.0, 1.0),
            comet_ecs::Anchor::BottomCenter => (0.5, 1.0),
            comet_ecs::Anchor::BottomRight => (1.0, 1.0),
        };
        let block_origin = v2::new(
            screen_position.x() - max_line_width * anchor_x,
            screen_position.y() + block_height * anchor_y,
        );

        let mut y_offset = 0.0f32;

        for (line, line_width) in lines.into_iter().zip(line_widths) {
            let mut x_offset = match justification {
                comet_ecs::TextJustification::Left => 0.0,
                comet_ecs::TextJustification::Center => (max_line_width - line_width) * 0.5,
                comet_ecs::TextJustification::Right => max_line_width - line_width,
            };
            for glyph in line {
                let region = glyph.region;
                let (dim_x, dim_y) = region.dimensions();
                let w = dim_x as f32 * glyph_scale;
                let h = dim_y as f32 * glyph_scale;
                let offset_x = region.offset_x() * glyph_scale;
                let offset_y = region.offset_y() * glyph_scale;
                let field = match glyph.representation {
                    GlyphRepresentation::Bitmap => 0.0,
                    GlyphRepresentation::Mtsdf => glyph.distance_range,
                    GlyphRepresentation::Pixel => -1.0,
                };

                let glyph_left = block_origin.x() + x_offset + offset_x;
                let glyph_top = block_origin.y() - offset_y - y_offset;
                let (glyph_left, glyph_top) = if glyph.representation == GlyphRepresentation::Pixel
                {
                    (glyph_left.round(), glyph_top.round())
                } else {
                    (glyph_left, glyph_top)
                };
                let glyph_right = glyph_left + w;
                let glyph_bottom = glyph_top - h;

                let buffer_size = vertex_data.len() as u16;
                vertex_data.extend_from_slice(&[
                    Vertex::new(
                        [glyph_left, glyph_top, field],
                        [region.u0(), region.v0()],
                        vert_color,
                    ),
                    Vertex::new(
                        [glyph_left, glyph_bottom, field],
                        [region.u0(), region.v1()],
                        vert_color,
                    ),
                    Vertex::new(
                        [glyph_right, glyph_bottom, field],
                        [region.u1(), region.v1()],
                        vert_color,
                    ),
                    Vertex::new(
                        [glyph_right, glyph_top, field],
                        [region.u1(), region.v0()],
                        vert_color,
                    ),
                ]);
                index_data.extend_from_slice(&[
                    buffer_size,
                    buffer_size + 1,
                    buffer_size + 3,
                    buffer_size + 1,
                    buffer_size + 2,
                    buffer_size + 3,
                ]);

                x_offset += region.advance() * glyph_scale;
            }

            y_offset += line_height;
        }
    }

    pub(super) fn resolve_text_size(
        &self,
        size: comet_ecs::TextSize,
        view: crate::camera::ResolvedCameraViewport,
    ) -> (comet_math::Px, f32) {
        let world_units_per_pixel =
            view.visible_world_size.y() / view.viewport.height.max(1) as f32;
        match size {
            comet_ecs::TextSize::Screen(size) => {
                let raster_size = size.resolve(self.scale_factor() as f32);
                (raster_size, world_units_per_pixel)
            }
            comet_ecs::TextSize::World(world_size) => {
                let raster_size = comet_math::px(world_size / world_units_per_pixel);
                (raster_size, world_units_per_pixel)
            }
        }
    }
}
