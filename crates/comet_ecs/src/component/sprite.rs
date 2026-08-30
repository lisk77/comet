use super::*;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Sprite {
    is_visible: bool,
    texture: ImageRef,
    draw_index: u32,
}

impl Sprite {
    pub fn with_texture(texture: impl Into<AssetSource<Image>>) -> Self {
        Self {
            is_visible: true,
            texture: texture.into().into(),
            draw_index: 0,
        }
    }

    pub fn draw_index(&self) -> u32 {
        self.draw_index
    }

    pub fn with_draw_index(mut self, index: u32) -> Self {
        self.draw_index = index;
        self
    }

    pub fn set_draw_index(&mut self, index: u32) {
        self.draw_index = index
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn with_visibility(mut self, is_visible: bool) -> Self {
        self.is_visible = is_visible;
        self
    }

    pub fn set_visibility(&mut self, is_visible: bool) {
        self.is_visible = is_visible;
    }

    pub fn texture(&self) -> ImageRef {
        self.texture.clone()
    }

    pub fn set_texture(&mut self, texture: impl Into<AssetSource<Image>>) {
        self.texture = texture.into().into();
    }

    pub fn set_image_ref(&mut self, image_ref: ImageRef) {
        self.texture = image_ref;
    }
}
