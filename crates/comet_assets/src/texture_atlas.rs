use crate::asset_handle::Asset;
use comet_app::resolve_asset_path;
use crate::font::GlyphData;
use crate::image::Image;
use comet_log::*;
use guillotiere::{size2, AllocId, AtlasAllocator};
use image::{DynamicImage, RgbaImage};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn u0(&self) -> f32 { self.u0 }
    pub fn u1(&self) -> f32 { self.u1 }
    pub fn v0(&self) -> f32 { self.v0 }
    pub fn v1(&self) -> f32 { self.v1 }
    pub fn dimensions(&self) -> (u32, u32) { self.dimensions }
    pub fn advance(&self) -> f32 { self.advance }
    pub fn offset_x(&self) -> f32 { self.offset_x }
    pub fn offset_y(&self) -> f32 { self.offset_y }
}

pub struct TextureAtlas {
    atlas: DynamicImage,
    allocator: AtlasAllocator,
    textures: HashMap<String, TextureRegion>,
    handle_textures: HashMap<Asset<Image>, (AllocId, TextureRegion, u64)>,
    width: u32,
    height: u32,
    current_frame: u64,
}

impl TextureAtlas {
    pub fn empty() -> Self {
        Self {
            atlas: DynamicImage::new_rgba8(1, 1),
            allocator: AtlasAllocator::new(size2(1, 1)),
            textures: HashMap::new(),
            handle_textures: HashMap::new(),
            width: 1,
            height: 1,
            current_frame: 0,
        }
    }

    /// Create a blank atlas pre-allocated to `size` squared pixels.
    /// Textures are inserted lazily at runtime via `insert_image_handle`.
    pub fn with_capacity(size: u32) -> Self {
        let size = size.max(64);
        Self {
            atlas: DynamicImage::new_rgba8(size, size),
            allocator: AtlasAllocator::new(size2(size as i32, size as i32)),
            textures: HashMap::new(),
            handle_textures: HashMap::new(),
            width: size,
            height: size,
            current_frame: 0,
        }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }

    pub fn texture_paths(&self) -> Vec<String> {
        self.textures.keys().cloned().collect()
    }

    #[inline(always)]
    fn next_power_of_two(mut x: u32) -> u32 {
        if x == 0 { return 1; }
        x -= 1;
        x |= x >> 1;
        x |= x >> 2;
        x |= x >> 4;
        x |= x >> 8;
        x |= x >> 16;
        x + 1
    }

    fn compute_region(x: u32, y: u32, w: u32, h: u32, atlas_w: u32, atlas_h: u32) -> TextureRegion {
        let aw = atlas_w as f32;
        let ah = atlas_h as f32;
        let eps = 0.01_f32;
        TextureRegion::new(
            (x as f32 + eps) / aw,
            (y as f32 + eps) / ah,
            (x as f32 + w as f32 - eps) / aw,
            (y as f32 + h as f32 - eps) / ah,
            (w, h),
            0.0, 0.0, 0.0,
        )
    }

    fn pack_textures_guillotiere(
        textures: &[(&String, &DynamicImage)],
        padding: u32,
    ) -> (u32, u32, AtlasAllocator, HashMap<String, (AllocId, u32, u32)>) {
        let total_area: u64 = textures.iter().map(|(_, t)| {
            let w = (t.width().max(1) + padding * 2) as u64;
            let h = (t.height().max(1) + padding * 2) as u64;
            w * h
        }).sum::<u64>().max(1);

        let min_side = ((total_area as f64).sqrt() as u32).max(64);
        let mut atlas_size = Self::next_power_of_two(min_side);
        let max_size = 8192u32;

        loop {
            let mut allocator = AtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32));
            let mut placements: HashMap<String, (AllocId, u32, u32)> = HashMap::new();
            let mut failed = false;

            for (name, tex) in textures {
                let w = (tex.width().max(1) + padding * 2) as i32;
                let h = (tex.height().max(1) + padding * 2) as i32;
                match allocator.allocate(size2(w, h)) {
                    Some(alloc) => {
                        let blit_x = alloc.rectangle.min.x as u32 + padding;
                        let blit_y = alloc.rectangle.min.y as u32 + padding;
                        placements.insert((*name).clone(), (alloc.id, blit_x, blit_y));
                    }
                    None => { failed = true; break; }
                }
            }

            if !failed {
                return (atlas_size, atlas_size, allocator, placements);
            }

            if atlas_size >= max_size {
                error!("Failed to pack all textures even at max atlas size ({max_size}x{max_size}).");
                return (atlas_size, atlas_size, allocator, placements);
            }

            atlas_size = (atlas_size * 2).min(max_size);
        }
    }

    fn build_atlas_from_placements(
        textures: &[(&String, &DynamicImage)],
        placements: &HashMap<String, (AllocId, u32, u32)>,
        atlas_w: u32,
        atlas_h: u32,
    ) -> (RgbaImage, HashMap<String, TextureRegion>) {
        let mut base = RgbaImage::new(atlas_w, atlas_h);
        let mut regions = HashMap::new();

        for (name, tex) in textures {
            if tex.width() == 0 || tex.height() == 0 { continue; }
            if let Some((_, blit_x, blit_y)) = placements.get(*name) {
                let rgba_owned;
                let rgba: &RgbaImage = if let Some(r) = tex.as_rgba8() { r } else {
                    rgba_owned = tex.to_rgba8();
                    &rgba_owned
                };
                Self::blit(&mut base, rgba, *blit_x, *blit_y);
                let region = Self::compute_region(*blit_x, *blit_y, tex.width(), tex.height(), atlas_w, atlas_h);
                regions.insert((*name).clone(), region);
            }
        }

        (base, regions)
    }

    fn blit(dst: &mut RgbaImage, src: &RgbaImage, x: u32, y: u32) {
        let src_stride = src.width() as usize * 4;
        let dst_stride = dst.width() as usize * 4;
        let dst_raw = dst.as_mut();
        let src_raw = src.as_raw();
        for row in 0..src.height() as usize {
            let src_off = row * src_stride;
            let dst_off = ((y as usize + row) * (dst_stride / 4) + x as usize) * 4;
            dst_raw[dst_off..dst_off + src_stride]
                .copy_from_slice(&src_raw[src_off..src_off + src_stride]);
        }
    }

    pub fn from_texture_paths(paths: Vec<String>) -> Self {
        let mut textures = Vec::new();

        info!("Loading textures...");
        for path in &paths {
            let img = match image::open(resolve_asset_path(path)) {
                Ok(i) => DynamicImage::ImageRgba8(i.into_rgba8()),
                Err(e) => { error!("Failed to load texture '{}': {}", path, e); continue; }
            };
            textures.push((path, img));
        }

        info!("Packing textures...");
        let tex_refs: Vec<(&String, &DynamicImage)> = textures.iter().map(|(p, i)| (*p, i)).collect();
        let (atlas_w, atlas_h, allocator, placements) = Self::pack_textures_guillotiere(&tex_refs, 2);
        let (base, regions) = Self::build_atlas_from_placements(&tex_refs, &placements, atlas_w, atlas_h);

        info!("Created texture atlas ({}x{}) with {} textures.", atlas_w, atlas_h, regions.len());

        TextureAtlas {
            atlas: DynamicImage::ImageRgba8(base),
            allocator,
            textures: regions,
            handle_textures: HashMap::new(),
            width: atlas_w,
            height: atlas_h,
            current_frame: 0,
        }
    }

    pub fn from_textures(names: Vec<String>, textures: Vec<DynamicImage>) -> Self {
        assert_eq!(names.len(), textures.len(), "Names and textures must have the same length.");

        let textures: Vec<DynamicImage> = textures.into_iter()
            .map(|t| if t.as_rgba8().is_some() { t } else { DynamicImage::ImageRgba8(t.into_rgba8()) })
            .collect();
        let tex_refs: Vec<(&String, &DynamicImage)> = names.iter().zip(textures.iter()).collect();

        let (atlas_w, atlas_h, allocator, placements) = Self::pack_textures_guillotiere(&tex_refs, 2);
        let (base, regions) = Self::build_atlas_from_placements(&tex_refs, &placements, atlas_w, atlas_h);

        TextureAtlas {
            atlas: DynamicImage::ImageRgba8(base),
            allocator,
            textures: regions,
            handle_textures: HashMap::new(),
            width: atlas_w,
            height: atlas_h,
            current_frame: 0,
        }
    }

    pub fn from_glyphs(glyphs: &[GlyphData]) -> Self {
        let tex_refs: Vec<(&String, &DynamicImage)> =
            glyphs.iter().map(|g| (&g.name, &g.render)).collect();

        let (atlas_w, atlas_h, allocator, placements) = Self::pack_textures_guillotiere(&tex_refs, 2);

        let mut base = RgbaImage::new(atlas_w, atlas_h);
        let mut regions = HashMap::new();

        for g in glyphs.iter() {
            if let Some((_, blit_x, blit_y)) = placements.get(&g.name) {
                let rgba_owned;
                let rgba: &RgbaImage = if let Some(r) = g.render.as_rgba8() { r } else {
                    rgba_owned = g.render.to_rgba8();
                    &rgba_owned
                };
                Self::blit(&mut base, rgba, *blit_x, *blit_y);

                let aw = atlas_w as f32;
                let ah = atlas_h as f32;
                let eps = 0.01_f32;
                let u0 = (*blit_x as f32 + eps) / aw;
                let v0 = (*blit_y as f32 + eps) / ah;
                let u1 = (*blit_x as f32 + g.render.width() as f32 - eps) / aw;
                let v1 = (*blit_y as f32 + g.render.height() as f32 - eps) / ah;

                regions.insert(g.name.clone(), TextureRegion::new(
                    u0, v0, u1, v1,
                    (g.render.width(), g.render.height()),
                    g.advance, g.offset_x, g.offset_y,
                ));
            }
        }

        TextureAtlas {
            atlas: DynamicImage::ImageRgba8(base),
            allocator,
            textures: regions,
            handle_textures: HashMap::new(),
            width: atlas_w,
            height: atlas_h,
            current_frame: 0,
        }
    }

    /// Allocate space for a runtime image handle in the atlas.
    pub fn insert_image_handle(
        &mut self,
        handle: Asset<Image>,
        w: u32,
        h: u32,
        pad: u32,
    ) -> Option<(u32, u32, TextureRegion)> {
        let alloc = self.allocator.allocate(size2((w + pad * 2) as i32, (h + pad * 2) as i32))?;
        let blit_x = alloc.rectangle.min.x as u32 + pad;
        let blit_y = alloc.rectangle.min.y as u32 + pad;
        let region = Self::compute_region(blit_x, blit_y, w, h, self.width, self.height);
        self.handle_textures.insert(handle, (alloc.id, region, self.current_frame));
        Some((blit_x, blit_y, region))
    }

    /// Immediately remove a handle from the atlas, freeing its allocated space.
    pub fn evict_handle(&mut self, handle: Asset<Image>) {
        if let Some((alloc_id, _, _)) = self.handle_textures.remove(&handle) {
            self.allocator.deallocate(alloc_id);
        }
    }

    /// Look up the UV region for a previously inserted handle.
    pub fn region_for_handle(&self, handle: Asset<Image>) -> Option<TextureRegion> {
        self.handle_textures.get(&handle).map(|(_, r, _)| *r)
    }

    /// Returns all handles currently tracked by the atlas (for rebuild re-packing).
    pub fn handle_keys(&self) -> Vec<Asset<Image>> {
        self.handle_textures.keys().cloned().collect()
    }

    /// Reset the allocator and handle map to new dimensions for a rebuild.
    /// String-keyed textures are also cleared since their UVs are no longer valid at the new size.
    pub fn reset_for_rebuild(&mut self, new_width: u32, new_height: u32) {
        self.allocator = AtlasAllocator::new(size2(new_width as i32, new_height as i32));
        self.handle_textures.clear();
        self.textures.clear();
        self.width = new_width;
        self.height = new_height;
        self.current_frame = 0;
    }

    /// Mark a handle as referenced in the current frame, resetting its eviction timer.
    pub fn mark_used(&mut self, handle: Asset<Image>) {
        if let Some((_, _, last_seen)) = self.handle_textures.get_mut(&handle) {
            *last_seen = self.current_frame;
        }
    }

    /// Advance the frame counter and deallocate handles not seen within `max_unseen_frames`.
    pub fn evict_stale(&mut self, max_unseen_frames: u64) {
        self.current_frame += 1;
        let threshold = self.current_frame.saturating_sub(max_unseen_frames);
        let to_evict: Vec<(Asset<Image>, AllocId)> = self.handle_textures
            .iter()
            .filter(|(_, (_, _, last_seen))| *last_seen < threshold)
            .map(|(h, (id, _, _))| (*h, *id))
            .collect();
        for (handle, alloc_id) in to_evict {
            self.allocator.deallocate(alloc_id);
            self.handle_textures.remove(&handle);
        }
    }

    pub fn atlas(&self) -> &DynamicImage {
        &self.atlas
    }

    /// Drop the CPU-side atlas image after it has been uploaded to the GPU.
    /// Only the texture region metadata is needed after upload.
    pub fn clear_atlas_image(&mut self) {
        self.atlas = DynamicImage::new_rgba8(1, 1);
    }

    pub fn textures(&self) -> &HashMap<String, TextureRegion> {
        &self.textures
    }
}
