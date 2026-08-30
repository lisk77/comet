use crate::Component;
use comet_assets::AssetId;

pub trait RenderAsset: Component {
    fn asset_id(&self) -> Option<AssetId>;
}
