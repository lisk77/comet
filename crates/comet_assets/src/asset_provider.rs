use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{AssetManager, Asset, Image, Font, TextureAtlas, AudioClip};
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

    pub fn with_image<F, R>(&self, handle: Asset<Image>, f: F) -> Option<R>
    where F: FnOnce(&Image) -> R {
        self.inner.write().ok().and_then(|mut m| m.get_image(handle).map(f))
    }

    pub fn with_font<F, R>(&self, handle: Asset<Font>, f: F) -> Option<R>
    where F: FnOnce(&Font) -> R {
        self.inner.write().ok().and_then(|mut m| m.get_font(handle).map(f))
    }

    pub fn with_texture_atlas<F, R>(&self, handle: Asset<TextureAtlas>, f: F) -> Option<R>
    where F: FnOnce(&TextureAtlas) -> R {
        self.inner.write().ok().and_then(|mut m| m.get_texture_atlas(handle).map(f))
    }

    pub fn with_audio_clip<F, R>(&self, handle: Asset<AudioClip>, f: F) -> Option<R>
    where F: FnOnce(&AudioClip) -> R {
        self.inner.write().ok().and_then(|mut m| m.get_audio_clip(handle).map(f))
    }

    pub fn add_image(&self, image: Image) -> Option<Asset<Image>> {
        self.inner.write().ok().map(|mut m| m.add_image(image))
    }

    pub fn add_font(&self, font: Font) -> Option<Asset<Font>> {
        self.inner.write().ok().map(|mut m| m.add_font(font))
    }

    pub fn add_texture_atlas(&self, atlas: TextureAtlas) -> Option<Asset<TextureAtlas>> {
        self.inner.write().ok().map(|mut m| m.add_texture_atlas(atlas))
    }

    pub fn add_audio_clip(&self, clip: AudioClip) -> Option<Asset<AudioClip>> {
        self.inner.write().ok().map(|mut m| m.add_audio_clip(clip))
    }

    pub fn remove_image(&self, handle: Asset<Image>) -> Option<Image> {
        self.inner.write().ok().and_then(|mut m| m.remove_image(handle))
    }

    pub fn remove_font(&self, handle: Asset<Font>) -> Option<Font> {
        self.inner.write().ok().and_then(|mut m| m.remove_font(handle))
    }

    pub fn remove_texture_atlas(&self, handle: Asset<TextureAtlas>) -> Option<TextureAtlas> {
        self.inner.write().ok().and_then(|mut m| m.remove_texture_atlas(handle))
    }

    pub fn remove_audio_clip(&self, handle: Asset<AudioClip>) -> Option<AudioClip> {
        self.inner.write().ok().and_then(|mut m| m.remove_audio_clip(handle))
    }

    /// Register a loader for a file extension.
    pub fn register_loader<T: Loadable>(
        &self,
        ext: impl Into<String>,
        loader: impl Fn(&[u8], &str) -> Result<T, anyhow::Error> + Send + Sync + 'static,
    ) {
        if let Ok(mut m) = self.inner.write() {
            m.register_loader(ext, loader);
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

        let (index, generation, worker) = match self.inner.write() {
            Ok(mut manager) => match manager.get_alloc_loader_typed::<T>(ext) {
                Some(alloc) => alloc(&mut *manager),
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
                    // worker dropped here — channel closes, slot transitions to Failed on next access
                }
            }
            ready.fetch_add(1, Ordering::Relaxed);
        });

        handle
    }

    /// Non-blocking load state for a typed handle.
    pub fn load_state<T: Loadable>(&self, handle: Asset<T>) -> LoadState {
        self.inner.write().ok()
            .map(|mut m| T::load_state(handle, &mut m))
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
