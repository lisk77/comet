use crate::font::*;
use comet_log::*;
use image::{DynamicImage, GenericImage, GenericImageView, RgbaImage};
use rect_packer::{Config, Packer, Rect};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TextureRegion {
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    advance: f32,
    offset_x: f32,
    offset_y: f32,
    dimensions: (u32, u32),
}

impl TextureRegion {
    pub fn new(
        u0: f32,
        v0: f32,
        u1: f32,
        v1: f32,
        dimensions: (u32, u32),
        advance: f32,
        offset_x: f32,
        offset_y: f32,
    ) -> Self {
        Self {
            u0,
            v0,
            u1,
            v1,
            advance,
            offset_x,
            offset_y,
            dimensions,
        }
    }

    pub fn u0(&self) -> f32 {
        self.u0
    }

    pub fn u1(&self) -> f32 {
        self.u1
    }

    pub fn v0(&self) -> f32 {
        self.v0
    }

    pub fn v1(&self) -> f32 {
        self.v1
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    pub fn advance(&self) -> f32 {
        self.advance
    }

    pub fn offset_x(&self) -> f32 {
        self.offset_x
    }

    pub fn offset_y(&self) -> f32 {
        self.offset_y
    }
}

#[derive(Debug, Clone)]
pub struct TextureAtlas {
    atlas: DynamicImage,
    textures: HashMap<String, TextureRegion>,
}

impl TextureAtlas {
    pub fn empty() -> Self {
        Self {
            atlas: DynamicImage::new_rgba8(1, 1),
            textures: HashMap::new(),
        }
    }

    pub fn texture_paths(&self) -> Vec<String> {
        self.textures.keys().cloned().collect()
    }

    #[inline(always)]
    fn next_power_of_two(mut x: u32) -> u32 {
        if x == 0 {
            return 1;
        }
        x -= 1;
        x |= x >> 1;
        x |= x >> 2;
        x |= x >> 4;
        x |= x >> 8;
        x |= x >> 16;
        x + 1
    }

    fn pack_textures(
        textures: &[(&String, &DynamicImage)],
        padding: u32,
    ) -> (u32, u32, HashMap<String, Rect>) {
        let mut atlas_size = 512;
        let max_size = 8192;

        let valid_textures: Vec<(String, DynamicImage)> = textures
            .iter()
            .map(|(name, tex)| {
                let (w, h) = (tex.width(), tex.height());
                if w == 0 || h == 0 {
                    warn!(
                        "Texture '{}' has invalid size {}x{}, replacing with 1x1 transparent dummy.",
                        name, w, h
                    );
                    let mut img = RgbaImage::new(1, 1);
                    img.put_pixel(0, 0, image::Rgba([0, 0, 0, 0]));
                    (name.to_string(), DynamicImage::ImageRgba8(img))
                } else {
                    ((*name).clone(), (*tex).clone())
                }
            })
            .collect();

        if valid_textures.is_empty() {
            error!("No valid textures to pack!");
            return (0, 0, HashMap::new());
        }

        loop {
            let config = Config {
                width: atlas_size as i32,
                height: atlas_size as i32,
                border_padding: padding as i32,
                rectangle_padding: padding as i32,
            };

            let mut packer = Packer::new(config);
            let mut placements = HashMap::new();
            let mut max_x = 0i32;
            let mut max_y = 0i32;
            let mut failed = false;

            for (name, tex) in &valid_textures {
                let width = tex.width() as i32;
                let height = tex.height() as i32;

                if width > atlas_size as i32 || height > atlas_size as i32 {
                    error!(
                        "Texture '{}' is too large ({width}x{height}) for current atlas size {atlas_size}x{atlas_size}",
                        name
                    );
                    failed = true;
                    break;
                }

                if let Some(rect) = packer.pack(width, height, false) {
                    max_x = max_x.max(rect.x + rect.width);
                    max_y = max_y.max(rect.y + rect.height);
                    placements.insert(name.clone(), rect);
                } else {
                    failed = true;
                    break;
                }
            }

            if failed {
                if atlas_size >= max_size {
                    error!(
                        "Failed to pack all textures even at max atlas size ({}x{}).",
                        max_size, max_size
                    );
                    return (max_x as u32, max_y as u32, placements);
                }

                info!(
                    "Atlas size {}x{} too small, doubling to {}x{}...",
                    atlas_size,
                    atlas_size,
                    atlas_size * 2,
                    atlas_size * 2
                );
                atlas_size *= 2;
            } else {
                info!(
                    "Created texture atlas ({}x{}) with {} textures.",
                    atlas_size,
                    atlas_size,
                    placements.len()
                );
                return (max_x as u32, max_y as u32, placements);
            }
        }
    }

    fn build_atlas(
        textures: &[(&String, &DynamicImage)],
        placements: &HashMap<String, Rect>,
        atlas_width: u32,
        atlas_height: u32,
    ) -> (RgbaImage, HashMap<String, TextureRegion>) {
        let mut base = RgbaImage::new(atlas_width, atlas_height);
        let mut regions = HashMap::new();

        for (name, tex) in textures {
            if let Some(rect) = placements.get(*name) {
                base.copy_from(&tex.to_rgba8(), rect.x as u32, rect.y as u32)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Failed to blit texture '{}' into atlas at ({}, {})",
                            name, rect.x, rect.y
                        )
                    });

                let u0 = rect.x as f32 / atlas_width as f32;
                let v0 = rect.y as f32 / atlas_height as f32;
                let u1 = (rect.x + rect.width) as f32 / atlas_width as f32;
                let v1 = (rect.y + rect.height) as f32 / atlas_height as f32;

                regions.insert(
                    (*name).clone(),
                    TextureRegion::new(
                        u0,
                        v0,
                        u1,
                        v1,
                        (rect.width as u32, rect.height as u32),
                        0.0,
                        0.0,
                        0.0,
                    ),
                );
            }
        }

        (base, regions)
    }

    pub fn from_texture_paths(paths: Vec<String>) -> Self {
        let mut textures = Vec::new();

        info!("Loading textures...");
        for path in &paths {
            let img = image::open(Path::new(path)).expect("Failed to load texture");
            textures.push((path, img));
        }

        info!("Packing textures...");
        let tex_refs: Vec<(&String, &DynamicImage)> =
            textures.iter().map(|(p, i)| (*p, i)).collect();

        let (atlas_w, atlas_h, placements) = Self::pack_textures(&tex_refs, 2);

        let atlas_w = Self::next_power_of_two(atlas_w);
        let atlas_h = Self::next_power_of_two(atlas_h);

        let (base, regions) = Self::build_atlas(&tex_refs, &placements, atlas_w, atlas_h);

        info!(
            "Created texture atlas ({}x{}) with {} textures.",
            atlas_w,
            atlas_h,
            regions.len()
        );

        TextureAtlas {
            atlas: DynamicImage::ImageRgba8(base),
            textures: regions,
        }
    }

    pub fn from_textures(names: Vec<String>, textures: Vec<DynamicImage>) -> Self {
        assert_eq!(
            names.len(),
            textures.len(),
            "Names and textures must have the same length."
        );

        let tex_refs: Vec<(&String, &DynamicImage)> = names.iter().zip(textures.iter()).collect();

        let (atlas_w, atlas_h, placements) = Self::pack_textures(&tex_refs, 2);
        let atlas_w = Self::next_power_of_two(atlas_w);
        let atlas_h = Self::next_power_of_two(atlas_h);

        let (base, regions) = Self::build_atlas(&tex_refs, &placements, atlas_w, atlas_h);

        TextureAtlas {
            atlas: DynamicImage::ImageRgba8(base),
            textures: regions,
        }
    }

    pub fn from_glyphs(glyphs: Vec<GlyphData>) -> Self {
        let textures: Vec<(String, DynamicImage)> = glyphs
            .iter()
            .map(|g| (g.name.clone(), g.render.clone()))
            .collect();

        let tex_refs: Vec<(&String, &DynamicImage)> =
            textures.iter().map(|(n, i)| (n, i)).collect();

        let (atlas_w, atlas_h, placements) = Self::pack_textures(&tex_refs, 2);
        let atlas_w = Self::next_power_of_two(atlas_w);
        let atlas_h = Self::next_power_of_two(atlas_h);

        let mut base = RgbaImage::new(atlas_w, atlas_h);
        let mut regions = HashMap::new();

        for g in glyphs.iter() {
            if let Some(rect) = placements.get(&g.name) {
                base.copy_from(&g.render.to_rgba8(), rect.x as u32, rect.y as u32)
                    .unwrap();

                let u0 = rect.x as f32 / atlas_w as f32;
                let v0 = rect.y as f32 / atlas_h as f32;
                let u1 = (rect.x + rect.width) as f32 / atlas_w as f32;
                let v1 = (rect.y + rect.height) as f32 / atlas_h as f32;

                let region = TextureRegion::new(
                    u0,
                    v0,
                    u1,
                    v1,
                    (rect.width as u32, rect.height as u32),
                    g.advance,
                    g.offset_x,
                    g.offset_y,
                );

                regions.insert(g.name.clone(), region);
            }
        }

        TextureAtlas {
            atlas: DynamicImage::ImageRgba8(base),
            textures: regions,
        }
    }

    pub fn from_fonts(fonts: &[Font]) -> Self {
        if fonts.is_empty() {
            return Self::empty();
        }

        let mut all_glyphs = Vec::new();

        for font in fonts {
            let font_name = font.name();
            let src_atlas = font.glyphs().atlas();
            let atlas_width = src_atlas.width();
            let atlas_height = src_atlas.height();

            for (glyph_name, region) in font.glyphs().textures() {
                let src_x = (region.u0() * atlas_width as f32) as u32;
                let src_y = (region.v0() * atlas_height as f32) as u32;
                let width = region.dimensions().0;
                let height = region.dimensions().1;

                let glyph_img = src_atlas.view(src_x, src_y, width, height).to_image();

                let key = format!("{}::{}", font_name, glyph_name);
                all_glyphs.push((key, DynamicImage::ImageRgba8(glyph_img), region.clone()));
            }
        }

        let tex_refs: Vec<(&String, &DynamicImage)> =
            all_glyphs.iter().map(|(n, i, _)| (n, i)).collect();
        let (atlas_w, atlas_h, placements) = Self::pack_textures(&tex_refs, 2);
        let atlas_w = Self::next_power_of_two(atlas_w);
        let atlas_h = Self::next_power_of_two(atlas_h);

        let mut base = RgbaImage::new(atlas_w, atlas_h);
        let mut regions = HashMap::new();

        for (key, img, original_region) in all_glyphs {
            if let Some(rect) = placements.get(&key) {
                base.copy_from(&img.to_rgba8(), rect.x as u32, rect.y as u32)
                    .unwrap();

                let u0 = rect.x as f32 / atlas_w as f32;
                let v0 = rect.y as f32 / atlas_h as f32;
                let u1 = (rect.x + rect.width) as f32 / atlas_w as f32;
                let v1 = (rect.y + rect.height) as f32 / atlas_h as f32;

                regions.insert(
                    key,
                    TextureRegion::new(
                        u0,
                        v0,
                        u1,
                        v1,
                        (rect.width as u32, rect.height as u32),
                        original_region.advance(),
                        original_region.offset_x(),
                        original_region.offset_y(),
                    ),
                );
            }
        }

        TextureAtlas {
            atlas: DynamicImage::ImageRgba8(base),
            textures: regions,
        }
    }

    pub fn atlas(&self) -> &DynamicImage {
        &self.atlas
    }

    pub fn textures(&self) -> &HashMap<String, TextureRegion> {
        &self.textures
    }
}
