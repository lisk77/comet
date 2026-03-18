use crate::{asset_handle::Asset, texture_atlas::{TextureAtlas, TextureRegion}};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasRef {
    region: TextureRegion,
    atlas: Asset<TextureAtlas>,
}

impl AtlasRef {
    pub fn new(region: TextureRegion, atlas: Asset<TextureAtlas>) -> Self {
        Self { region, atlas }
    }

    pub fn region(&self) -> TextureRegion {
        self.region
    }

    pub fn atlas(&self) -> Asset<TextureAtlas> {
        self.atlas
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageRef {
    Unresolved(&'static str),
    Atlas(AtlasRef),
}

impl Default for ImageRef {
    fn default() -> Self {
        Self::Unresolved("")
    }
}

impl From<&'static str> for ImageRef {
    fn from(path: &'static str) -> Self {
        Self::Unresolved(path)
    }
}
