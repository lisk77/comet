use std::any::TypeId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use notify::{EventKind, RecursiveMode, Watcher};
use crate::{AssetManager, Asset};
use crate::asset_manager::Loadable;
use crate::image::Image;
use crate::texture_atlas::TextureAtlas;
use crate::asset_path::file_extension;
use crate::asset_store::LoadState;

#[derive(Clone)]
struct ReloadEntry {
    original_path: String,
    ext: String,
    type_id: TypeId,
    index: u32,
    generation: u32,
}

pub struct AssetProvider {
    inner: Arc<RwLock<AssetManager>>,
    queued: Arc<AtomicUsize>,
    ready: Arc<AtomicUsize>,
    reload_map: Arc<RwLock<HashMap<PathBuf, ReloadEntry>>>,
    _watcher: Arc<Mutex<Option<notify::RecommendedWatcher>>>,
}

impl AssetProvider {
    pub fn new(manager: AssetManager) -> Self {
        let inner = Arc::new(RwLock::new(manager));
        let queued = Arc::new(AtomicUsize::new(0));
        let ready = Arc::new(AtomicUsize::new(0));
        let reload_map: Arc<RwLock<HashMap<PathBuf, ReloadEntry>>> = Arc::new(RwLock::new(HashMap::new()));

        let watcher = Self::start_hot_reload(
            Arc::clone(&inner),
            Arc::clone(&reload_map),
            Arc::clone(&queued),
            Arc::clone(&ready),
        );

        Self { inner, queued, ready, reload_map, _watcher: Arc::new(Mutex::new(watcher)) }
    }

    fn start_hot_reload(
        inner: Arc<RwLock<AssetManager>>,
        reload_map: Arc<RwLock<HashMap<PathBuf, ReloadEntry>>>,
        queued: Arc<AtomicUsize>,
        ready: Arc<AtomicUsize>,
    ) -> Option<notify::RecommendedWatcher> {
        let (event_tx, event_rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();

        let mut watcher = match notify::recommended_watcher(move |res| { let _ = event_tx.send(res); }) {
            Ok(w) => w,
            Err(e) => { comet_log::warn!("Hot reload unavailable: {}", e); return None; }
        };

        let asset_root = crate::asset_path::asset_root();
        if let Err(e) = watcher.watch(&asset_root, RecursiveMode::Recursive) {
            comet_log::warn!("Hot reload: failed to watch '{}': {}", asset_root.display(), e);
            return None;
        }

        std::thread::spawn(move || {
            for event in event_rx {
                let Ok(event) = event else { continue; };
                if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) { continue; }

                for resolved_path in event.paths {
                    let entry = {
                        let Ok(map) = reload_map.read() else { continue; };
                        map.get(&resolved_path).cloned()
                    };
                    let Some(entry) = entry else { continue; };

                    let bytes = match std::fs::read(&resolved_path) {
                        Ok(b) => b,
                        Err(e) => { comet_log::error!("Hot reload: failed to read '{}': {}", resolved_path.display(), e); continue; }
                    };

                    let worker = {
                        let Ok(mut manager) = inner.write() else { continue; };
                        if entry.type_id == TypeId::of::<Image>() {
                            let image_handle = Asset::<Image>::new(entry.index, entry.generation);
                            manager.for_each_ready_mut::<TextureAtlas>(|atlas| {
                                atlas.evict_handle(image_handle);
                            });
                        }
                        manager.begin_reload(&entry.ext, entry.type_id, entry.index)
                    };
                    let Some(worker) = worker else { continue; };

                    comet_log::info!("Hot reloading '{}'", entry.original_path);
                    queued.fetch_add(1, Ordering::Relaxed);
                    let ready = Arc::clone(&ready);
                    let original_path = entry.original_path.clone();
                    std::thread::spawn(move || {
                        worker(bytes, original_path);
                        ready.fetch_add(1, Ordering::Relaxed);
                    });
                }
            }
        });

        Some(watcher)
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
    pub fn load<T: Loadable>(&self, path: &str) -> Asset<T> {
        let resolved = crate::asset_path::resolve_asset_path(path);

        let ext = match file_extension(&resolved, path) {
            Ok(e) => e,
            Err(e) => { comet_log::error!("{}", e); return Asset::default(); }
        };

        let (index, generation, worker) = match self.inner.write() {
            Ok(mut manager) => match manager.get_alloc_loader_typed::<T>(ext) {
                Some(alloc) => {
                    let result = alloc(&mut *manager);
                    manager.record_path::<T>(result.0, result.1, path);
                    result
                }
                None => {
                    comet_log::error!("No loader registered for '{}' producing the requested type", ext);
                    return Asset::default();
                }
            },
            Err(_) => { comet_log::error!("AssetManager lock poisoned"); return Asset::default(); }
        };

        if let Ok(mut map) = self.reload_map.write() {
            map.insert(resolved.clone(), ReloadEntry {
                original_path: path.to_string(),
                ext: ext.to_string(),
                type_id: TypeId::of::<T>(),
                index,
                generation,
            });
        }

        let handle = Asset::<T>::new(index, generation);
        self.queued.fetch_add(1, Ordering::Relaxed);
        let ready = Arc::clone(&self.ready);
        let original_path = path.to_string();

        std::thread::spawn(move || {
            match std::fs::read(&resolved) {
                Ok(bytes) => worker(bytes, original_path),
                Err(e) => comet_log::error!("Failed to read asset '{}': {}", resolved.display(), e),
            }
            ready.fetch_add(1, Ordering::Relaxed);
        });

        handle
    }

    /// Registers a handle (created via `add`) for hot reload watching.
    pub fn track_for_reload<T: Loadable>(&self, handle: Asset<T>, path: &str) {
        let resolved = crate::asset_path::resolve_asset_path(path);
        let ext = match file_extension(&resolved, path) {
            Ok(e) => e,
            Err(e) => { comet_log::error!("{}", e); return; }
        };
        if let Ok(mut map) = self.reload_map.write() {
            map.insert(resolved.clone(), ReloadEntry {
                original_path: path.to_string(),
                ext: ext.to_string(),
                type_id: TypeId::of::<T>(),
                index: handle.index(),
                generation: handle.generation(),
            });
        }
    }

    /// Finds a previously loaded asset by its original load path.
    pub fn find_by_path<T: Loadable>(&self, path: &str) -> Option<Asset<T>> {
        self.inner.read().ok().and_then(|m| m.find_by_path::<T>(path))
    }

    /// Finds a previously loaded asset by the stem of its original path.
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
            reload_map: Arc::clone(&self.reload_map),
            _watcher: Arc::clone(&self._watcher),
        }
    }
}
