use crate::Loadable;

pub trait AssetSettings: Clone + Send + Sync + 'static {
    type Asset: Loadable;

    fn load(&self, bytes: &[u8], path: &str) -> anyhow::Result<Self::Asset>;
}
