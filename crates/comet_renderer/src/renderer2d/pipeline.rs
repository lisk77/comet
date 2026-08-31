use super::shaders::{GIZMO_SHADER, SPRITE_SHADER};
use super::*;

impl Renderer2D {
    pub(super) fn setup_atlas_pipeline(&mut self, mut atlas: comet_assets::TextureAtlas) {
        let gpu_texture = match GpuTexture::from_dynamic_image(
            self.render_state.device(),
            self.render_state.queue(),
            atlas.atlas(),
            Some("Atlas"),
            false,
        ) {
            Ok(tex) => tex,
            Err(e) => {
                error!("Failed to convert atlas to GPU texture: {}", e);
                return;
            }
        };
        atlas.clear_atlas_image();

        if let Some(handle) = self.asset_provider.add(atlas) {
            self.render_state
                .resources_mut()
                .insert_asset_atlas_handle("atlas".to_string(), handle);
        } else {
            error!("Failed to add texture atlas to asset provider");
            return;
        }

        let gpu_texture_arc = Arc::new(gpu_texture);
        self.render_state
            .resources_mut()
            .insert_gpu_texture("atlas".to_string(), gpu_texture_arc.clone());

        let format = self.render_state.config().format;
        let width = self.render_state.config().width;
        let height = self.render_state.config().height;

        let sprite_vertex_contract = vec![
            MeshVertexAttribute::new(
                comet_ecs::VertexSemantic::Position,
                comet_ecs::VertexFormat::Float32x3,
                0,
            ),
            MeshVertexAttribute::new(
                comet_ecs::VertexSemantic::TexCoord(0),
                comet_ecs::VertexFormat::Float32x2,
                1,
            ),
        ];

        self.graph.add_node(
            PassNode::with_meshes(
                "Universal",
                SPRITE_SHADER,
                wgpu::PrimitiveTopology::TriangleList,
                Some(gpu_texture_arc),
                vec![],
                LoadOp::Background,
                sprite_vertex_contract,
                SpriteInstance::desc(),
                1024,
            ),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );

        self.graph.add_node(
            PassNode::new(
                "Gizmo",
                GIZMO_SHADER,
                wgpu::PrimitiveTopology::LineList,
                None,
                vec!["Universal", "Font"],
                LoadOp::Load,
            ),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );
    }
    pub(super) fn ensure_image_in_atlas(
        &mut self,
        handle: comet_assets::Asset<comet_assets::Image>,
    ) -> Option<AtlasRef> {
        let atlas_handle = self
            .render_state
            .resources()
            .get_asset_atlas_handle("atlas")?;

        if let Some(region) = self
            .asset_provider
            .with(atlas_handle, |atlas| atlas.region_for_handle(handle))
            .flatten()
        {
            return Some(AtlasRef::new(region, atlas_handle));
        }

        let (w, h) = self
            .asset_provider
            .with(handle, |img| (img.width(), img.height()))?;

        let alloc = self
            .asset_provider
            .with_mut(atlas_handle, |atlas| {
                atlas.insert_image_handle(handle, w, h, 1)
            })
            .flatten();

        let (blit_x, blit_y, region) = match alloc {
            Some(r) => r,
            None => {
                self.rebuild_atlas(atlas_handle);
                match self
                    .asset_provider
                    .with_mut(atlas_handle, |atlas| {
                        atlas.insert_image_handle(handle, w, h, 1)
                    })
                    .flatten()
                {
                    Some(r) => r,
                    None => {
                        error!("Failed to insert into atlas even after rebuild");
                        return None;
                    }
                }
            }
        };

        let gpu_texture = self
            .render_state
            .resources()
            .get_gpu_texture("atlas")?
            .clone();
        self.asset_provider.with(handle, |img| {
            gpu_texture.write_region(self.render_state.queue(), blit_x, blit_y, img.data(), w, h);
        });
        self.asset_provider
            .with_mut(handle, |img| img.evict_pixels());

        Some(AtlasRef::new(region, atlas_handle))
    }

    fn rebuild_atlas(&mut self, atlas_handle: comet_assets::Asset<comet_assets::TextureAtlas>) {
        let handles = self
            .asset_provider
            .with(atlas_handle, |atlas| atlas.handle_keys())
            .unwrap_or_default();
        let (old_w, old_h) = self
            .asset_provider
            .with(atlas_handle, |atlas| (atlas.width(), atlas.height()))
            .unwrap_or((512, 512));

        let new_size = (old_w * 2).max(old_h * 2).min(8192);
        info!(
            "Atlas full: rebuilding {}x{} → {}x{}",
            old_w, old_h, new_size, new_size
        );

        self.asset_provider.with_mut(atlas_handle, |atlas| {
            atlas.reset_for_rebuild(new_size, new_size);
        });

        let new_gpu = GpuTexture::create_2d_texture(
            self.render_state.device(),
            new_size,
            new_size,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            wgpu::FilterMode::Nearest,
            Some("Atlas"),
        );

        for h in handles {
            let dims = self
                .asset_provider
                .with(h, |img| (img.width(), img.height()));
            let Some((w, h_px)) = dims else {
                continue;
            };

            let result = self
                .asset_provider
                .with_mut(atlas_handle, |atlas| {
                    atlas.insert_image_handle(h, w, h_px, 1)
                })
                .flatten();
            let Some((blit_x, blit_y, _)) = result else {
                error!("Failed to re-pack handle during atlas rebuild");
                continue;
            };

            let uploaded = self
                .asset_provider
                .with(h, |img| {
                    if !img.is_evicted() {
                        new_gpu.write_region(
                            self.render_state.queue(),
                            blit_x,
                            blit_y,
                            img.data(),
                            w,
                            h_px,
                        );
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false);

            if !uploaded {
                let path = self.asset_provider.path_for::<comet_assets::Image>(h);
                if let Some(path) = path {
                    let fs_path = comet_assets::resolve_asset_path(&path);
                    if let Ok(bytes) = std::fs::read(&fs_path) {
                        if let Ok(img) = comet_assets::Image::from_bytes(&bytes, false) {
                            new_gpu.write_region(
                                self.render_state.queue(),
                                blit_x,
                                blit_y,
                                img.data(),
                                w,
                                h_px,
                            );
                        }
                    }
                }
            }
        }

        let new_gpu_arc = Arc::new(new_gpu);
        self.render_state
            .resources_mut()
            .insert_gpu_texture("atlas".to_string(), new_gpu_arc.clone());

        let device = self.render_state.device();
        if let Some(node) = self.graph.pass_mut("Universal") {
            node.set_texture(new_gpu_arc, device);
        }

        let _ = self.event_sender.send(Renderer2DEvent::AtlasRebuilt);
    }

    pub(super) fn add_pass(&mut self, desc: crate::render_commands::PassDescriptor) -> PassOutput {
        let load = if desc.render_target.is_some() {
            if let LoadOp::Color(_) | LoadOp::Background = desc.load {
                warn!(
                    "pass '{}': render_target with non-Load op, forcing Load",
                    desc.label
                );
            }
            LoadOp::Load
        } else {
            desc.load
        };

        let pass_output = PassOutput(desc.output.clone().unwrap_or_else(|| desc.label.clone()));

        let format = self.render_state.config().format;
        let width = self.render_state.config().width;
        let height = self.render_state.config().height;

        self.graph.add_node(
            PostProcessNode::new(
                desc.label,
                desc.inputs,
                desc.output,
                desc.render_target,
                desc.output_format,
                load,
                desc.shader_src,
            ),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );

        #[cfg(feature = "comet_debug")]
        info!("Created pass {}!", pass_output.name());
        pass_output
    }

    pub(super) fn remove_pass(&mut self, label: &str) {
        self.graph.remove_node(label);
    }

    pub(super) fn set_pass_render_target(&mut self, label: &str, render_target: Option<String>) {
        if let Some(node) = self.graph.post_process_mut(label) {
            node.set_render_target(render_target);
            self.graph.mark_dirty();
        } else {
            error!("set_pass_render_target: no PostProcessNode '{}'", label);
        }
    }

    pub(super) fn set_pass_output(
        &mut self,
        label: &str,
        output: Option<PassOutput>,
    ) -> Option<PassOutput> {
        if let Some(node) = self.graph.post_process_mut(label) {
            let result = output.clone();
            node.set_output(output.map(|p| p.0));
            self.graph.mark_dirty();
            result
        } else {
            error!("set_pass_output: no PostProcessNode '{}'", label);
            None
        }
    }

    pub(super) fn get_texture_region(&self, texture: AtlasRef) -> TextureRegion {
        texture.region()
    }
}
