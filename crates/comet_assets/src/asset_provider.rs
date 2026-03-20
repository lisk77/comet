use std::sync::{Arc, RwLock};
use crate::{AssetManager, Asset, Image, Font, TextureAtlas, AudioClip};

/// Thread-safe API for accessing assets from the `AssetManager`.
pub struct AssetProvider {
    inner: Arc<RwLock<AssetManager>>,
}

impl AssetProvider {
    /// Create a new AssetProvider from an AssetManager.
    pub fn new(manager: AssetManager) -> Self {
        Self {
            inner: Arc::new(RwLock::new(manager)),
        }
    }

    /// Access an image through a callback within a read lock.
    pub fn with_image<F, T>(&self, handle: Asset<Image>, f: F) -> Option<T>
    where
        F: FnOnce(&Image) -> T,
    {
        self.inner
            .read()
            .ok()
            .and_then(|manager| manager.get_image(handle).map(f))
    }

    /// Access a font through a callback within a read lock.
    pub fn with_font<F, T>(&self, handle: Asset<Font>, f: F) -> Option<T>
    where
        F: FnOnce(&Font) -> T,
    {
        self.inner
            .read()
            .ok()
            .and_then(|manager| manager.get_font(handle).map(f))
    }

    /// Access a texture atlas through a callback within a read lock.
    pub fn with_texture_atlas<F, T>(&self, handle: Asset<TextureAtlas>, f: F) -> Option<T>
    where
        F: FnOnce(&TextureAtlas) -> T,
    {
        self.inner
            .read()
            .ok()
            .and_then(|manager| manager.get_texture_atlas(handle).map(f))
    }

    /// Add an image to the asset store.
    pub fn add_image(&self, image: Image) -> Option<Asset<Image>> {
        self.inner.write().ok().map(|mut manager| manager.add_image(image))
    }

    /// Add a font to the asset store.
    pub fn add_font(&self, font: Font) -> Option<Asset<Font>> {
        self.inner.write().ok().map(|mut manager| manager.add_font(font))
    }

    /// Add a texture atlas to the asset store.
    pub fn add_texture_atlas(&self, atlas: TextureAtlas) -> Option<Asset<TextureAtlas>> {
        self.inner.write().ok().map(|mut manager| manager.add_texture_atlas(atlas))
    }

    /// Remove an image from the asset store.
    pub fn remove_image(&self, handle: Asset<Image>) -> Option<Image> {
        self.inner.write().ok().and_then(|mut manager| manager.remove_image(handle))
    }

    /// Remove a font from the asset store.
    pub fn remove_font(&self, handle: Asset<Font>) -> Option<Font> {
        self.inner.write().ok().and_then(|mut manager| manager.remove_font(handle))
    }

    /// Remove a texture atlas from the asset store.
    pub fn remove_texture_atlas(&self, handle: Asset<TextureAtlas>) -> Option<TextureAtlas> {
        self.inner.write().ok().and_then(|mut manager| manager.remove_texture_atlas(handle))
    }

    /// Access an audio clip through a callback within a read lock.
    pub fn with_audio_clip<F, T>(&self, handle: Asset<AudioClip>, f: F) -> Option<T>
    where
        F: FnOnce(&AudioClip) -> T,
    {
        self.inner
            .read()
            .ok()
            .and_then(|manager| manager.get_audio_clip(handle).map(f))
    }

    /// Add an audio clip to the asset store.
    pub fn add_audio_clip(&self, clip: AudioClip) -> Option<Asset<AudioClip>> {
        self.inner.write().ok().map(|mut manager| manager.add_audio_clip(clip))
    }

    /// Remove an audio clip from the asset store.
    pub fn remove_audio_clip(&self, handle: Asset<AudioClip>) -> Option<AudioClip> {
        self.inner.write().ok().and_then(|mut manager| manager.remove_audio_clip(handle))
    }

    /// Get a clone of `AssetManager` Arc.
    #[allow(unused)]
    pub(crate) fn inner_arc(&self) -> Arc<RwLock<AssetManager>> {
        Arc::clone(&self.inner)
    }
}

impl Clone for AssetProvider {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
