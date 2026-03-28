use std::sync::Arc;
use comet_macros::module;
use comet_app::{App, Module};
use crate::{Asset, AssetManager, AssetProvider, Loadable, LoadState};

pub struct AssetModule {
    provider: Arc<AssetProvider>,
}

impl AssetModule {
    pub fn new() -> Self {
        Self {
            provider: Arc::new(AssetProvider::new(AssetManager::new())),
        }
    }
}

impl Module for AssetModule {
    fn build(&mut self, app: &mut App) {
        app.add_context((*self.provider).clone());
    }
}

#[module]
impl AssetModule {
    pub fn asset_provider(&self) -> Arc<AssetProvider> {
        self.provider.clone()
    }

    pub fn load<A: Loadable>(&self, path: &str) -> Asset<A> {
        self.provider.load::<A>(path)
    }

    pub fn load_assets<A: Loadable>(&self, paths: Vec<&str>) -> Vec<Asset<A>> {
        paths.into_iter().map(|p| self.provider.load::<A>(p)).collect()
    }

    pub fn unload<A: Loadable>(&self, handle: Asset<A>) -> Option<A> {
        self.provider.unload(handle)
    }

    pub fn unload_assets<A: Loadable>(&self, handles: Vec<Asset<A>>) -> Vec<Option<A>> {
        self.provider.unload_assets(handles)
    }

    pub fn load_state<T: Loadable>(&self, handle: Asset<T>) -> LoadState {
        self.provider.load_state(handle)
    }

    pub fn load_progress(&self) -> (usize, usize) {
        self.provider.load_progress()
    }

    pub fn all_loaded(&self) -> bool {
        self.provider.all_loaded()
    }
}
