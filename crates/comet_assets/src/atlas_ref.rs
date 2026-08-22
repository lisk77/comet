use crate::{
    asset_handle::Asset,
    image::Image,
    texture_atlas::{TextureAtlas, TextureRegion},
    AssetPath, AssetSource,
};

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

#[derive(Debug, Clone, PartialEq)]
pub enum ImageRef {
    Unresolved(AssetPath),
    Atlas(AtlasRef),
    Handle(Asset<Image>),
    ResolvedHandle(Asset<Image>, AtlasRef),
}

impl Default for ImageRef {
    fn default() -> Self {
        Self::Unresolved(AssetPath::from(""))
    }
}

impl From<AssetSource<Image>> for ImageRef {
    fn from(source: AssetSource<Image>) -> Self {
        match source {
            AssetSource::Path(path) => Self::Unresolved(path),
            AssetSource::Handle(handle) => Self::Handle(handle),
        }
    }
}

impl From<AssetPath> for ImageRef {
    fn from(path: AssetPath) -> Self {
        Self::Unresolved(path)
    }
}

impl From<&str> for ImageRef {
    fn from(path: &str) -> Self {
        Self::Unresolved(path.into())
    }
}

impl From<String> for ImageRef {
    fn from(path: String) -> Self {
        Self::Unresolved(path.into())
    }
}

impl From<Asset<Image>> for ImageRef {
    fn from(handle: Asset<Image>) -> Self {
        Self::Handle(handle)
    }
}

impl From<AtlasRef> for ImageRef {
    fn from(atlas: AtlasRef) -> Self {
        Self::Atlas(atlas)
    }
}
