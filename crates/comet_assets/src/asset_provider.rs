use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{AssetManager, Asset};
use crate::asset_manager::Loadable;
use crate::asset_path::file_extension;
use crate::asset_store::LoadState;

pub struct AssetProvider {
    inner: Arc<RwLock<AssetManager>>,
    queued: Arc<AtomicUsize>,
    ready: Arc<AtomicUsize>,
}

impl AssetProvider {
    pub fn new(manager: AssetManager) -> Self {
        Self {
            inner: Arc::new(RwLock::new(manager)),
            queued: Arc::new(AtomicUsize::new(0)),
            ready: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn with<T: Loadable, F, R>(&self, handle: Asset<T>, f: F) -> Option<R>
    where F: FnOnce(&T) -> R {
        self.inner.write().ok().and_then(|mut m| m.get(handle).map(f))
    }

    pub fn with_mut<T: Loadable, F, R>(&self, handle: Asset<T>, f: F) -> Option<R>
    where F: FnOnce(&mut T) -> R {
        self.inner.write().ok().and_then(|mut m| m.get_mut(handle).map(f))
    }

    pub fn add<T: Loadable>(&self, asset: T) -> Option<Asset<T>> {
        self.inner.write().ok().map(|mut m| m.add(asset))
    }

    pub fn remove<T: Loadable>(&self, handle: Asset<T>) -> Option<T> {
        self.inner.write().ok().and_then(|mut m| m.remove(handle))
    }

    /// Register a loader for a file extension.
    /// Automatically registers `T` as an asset type. 
    pub fn register_loader<T: Loadable>(
        &self,
        ext: impl Into<String>,
        loader: impl Fn(&[u8], &str) -> Result<T, anyhow::Error> + Send + Sync + 'static,
    ) {
        if let Ok(mut m) = self.inner.write() {
            m.register_loader(ext, loader);
        }
    }

    /// Register a store for a type with no file loader.
    pub fn register_asset_type<T: Loadable>(&self) {
        if let Ok(mut m) = self.inner.write() {
            m.register_asset_type::<T>();
        }
    }

    /// Loads an asset from `path` in the background. Returns a typed handle immediately.
    /// Check progress with `load_state`. On any error the handle is in `Failed` state.
    pub fn load<T: Loadable>(&self, path: &str) -> Asset<T> {
        let resolved = crate::asset_path::resolve_asset_path(path);

        let ext = match file_extension(&resolved, path) {
            Ok(e) => e,
            Err(e) => { comet_log::error!("{}", e); return Asset::default(); }
        };

        let stem = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(path)
            .to_string();

        let (index, generation, worker) = match self.inner.write() {
            Ok(mut manager) => match manager.get_alloc_loader_typed::<T>(ext) {
                Some(alloc) => {
                    let result = alloc(&mut *manager);
                    manager.record_path::<T>(result.0, result.1, &stem);
                    result
                }
                None => {
                    comet_log::error!("No loader registered for '{}' producing the requested type", ext);
                    return Asset::default();
                }
            },
            Err(_) => { comet_log::error!("AssetManager lock poisoned"); return Asset::default(); }
        };

        let handle = Asset::<T>::new(index, generation);

        self.queued.fetch_add(1, Ordering::Relaxed);
        let ready = self.ready.clone();
        let original_path = path.to_string();

        std::thread::spawn(move || {
            match std::fs::read(&resolved) {
                Ok(bytes) => worker(bytes, original_path),
                Err(e) => {
                    comet_log::error!("Failed to read asset '{}': {}", resolved.display(), e);
                }
            }
            ready.fetch_add(1, Ordering::Relaxed);
        });

        handle
    }

    /// Finds a previously loaded asset by the stem of its original path (e.g. `"hit"` for `"res://sounds/hit.ogg"`).
    pub fn find_by_stem<T: Loadable>(&self, stem: &str) -> Option<Asset<T>> {
        self.inner.read().ok().and_then(|m| m.find_by_stem::<T>(stem))
    }

    /// Non-blocking load state for a typed handle.
    pub fn load_state<T: Loadable>(&self, handle: Asset<T>) -> LoadState {
        self.inner.write().ok()
            .map(|mut m| m.load_state(handle))
            .unwrap_or(LoadState::Failed)
    }

    /// Returns (assets_ready, assets_queued) — useful for a loading screen progress bar.
    pub fn load_progress(&self) -> (usize, usize) {
        (self.ready.load(Ordering::Relaxed), self.queued.load(Ordering::Relaxed))
    }

    /// Returns true when all queued background loads have finished (or there are none).
    pub fn all_loaded(&self) -> bool {
        let queued = self.queued.load(Ordering::Relaxed);
        queued == 0 || self.ready.load(Ordering::Relaxed) >= queued
    }

    #[allow(unused)]
    pub(crate) fn inner_arc(&self) -> Arc<RwLock<AssetManager>> {
        Arc::clone(&self.inner)
    }
}

impl Clone for AssetProvider {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            queued: Arc::clone(&self.queued),
            ready: Arc::clone(&self.ready),
        }
    }
}
