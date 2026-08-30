use crate::Component;
use comet_assets::{AssetId, AssetProvider};

pub trait RenderAsset: Component {
    fn resolve_asset(&mut self, assets: &AssetProvider) -> AssetId;
}
