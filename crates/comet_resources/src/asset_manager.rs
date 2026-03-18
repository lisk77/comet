use crate::{asset_store::*, asset_handle::*, image::Image, font::Font, texture_atlas::TextureAtlas};

pub struct AssetManager {
    images: AssetStore<Image>,
    fonts: AssetStore<Font>,
    texture_atlases: AssetStore<TextureAtlas>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            images: AssetStore::new(),
            fonts: AssetStore::new(),
            texture_atlases: AssetStore::new(),
        }
    }

    pub fn add_image(&mut self, image: Image) -> Asset<Image> {
        self.images.insert(image)
    }

    pub fn get_image(&self, handle: Asset<Image>) -> Option<&Image> {
        self.images.get(handle)
    }

    pub fn get_image_mut(&mut self, handle: Asset<Image>) -> Option<&mut Image> {
        self.images.get_mut(handle)
    }

    pub fn remove_image(&mut self, handle: Asset<Image>) -> Option<Image> {
        self.images.remove(handle)
    }
    
    pub fn add_font(&mut self, font: Font) -> Asset<Font> {
        self.fonts.insert(font)
    }

    pub fn get_font(&self, handle: Asset<Font>) -> Option<&Font> {
        self.fonts.get(handle)
    }

    pub fn get_font_mut(&mut self, handle: Asset<Font>) -> Option<&mut Font> {
        self.fonts.get_mut(handle)
    }

    pub fn remove_font(&mut self, handle: Asset<Font>) -> Option<Font> {
        self.fonts.remove(handle)
    }

    pub fn add_texture_atlas(&mut self, texture_atlas: TextureAtlas) -> Asset<TextureAtlas> {
        self.texture_atlases.insert(texture_atlas)
    }

    pub fn get_texture_atlas(&self, handle: Asset<TextureAtlas>) -> Option<&TextureAtlas> {
        self.texture_atlases.get(handle)
    }

    pub fn get_texture_atlas_mut(
        &mut self,
        handle: Asset<TextureAtlas>,
    ) -> Option<&mut TextureAtlas> {
        self.texture_atlases.get_mut(handle)
    }

    pub fn remove_texture_atlas(&mut self, handle: Asset<TextureAtlas>) -> Option<TextureAtlas> {
        self.texture_atlases.remove(handle)
    }
}
