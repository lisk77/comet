use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use crate::{asset_store::*, asset_handle::*, image::Image, font::Font, texture_atlas::TextureAtlas, audio_clip::AudioClip};
use crate::asset_path::resolve_asset_path;

/// A type-erased asset handle. Can be downcast to `Asset<T>` with `try_as`.
pub struct AnyHandle {
    index: u32,
    generation: u32,
    type_id: TypeId,
}

impl AnyHandle {
    pub fn try_as<T: 'static>(&self) -> Option<Asset<T>> {
        if self.type_id == TypeId::of::<T>() {
            Some(Asset::new(self.index, self.generation))
        } else {
            None
        }
    }
}

/// Trait for asset types that can be inserted into the `AssetManager`.
pub trait Loadable: Any + Send + Sync + 'static {
    fn insert_into(self, manager: &mut AssetManager) -> Asset<Self>
    where
        Self: Sized;
}

impl Loadable for Image {
    fn insert_into(self, manager: &mut AssetManager) -> Asset<Self> {
        manager.add_image(self)
    }
}

impl Loadable for Font {
    fn insert_into(self, manager: &mut AssetManager) -> Asset<Self> {
        manager.add_font(self)
    }
}

impl Loadable for TextureAtlas {
    fn insert_into(self, manager: &mut AssetManager) -> Asset<Self> {
        manager.add_texture_atlas(self)
    }
}

impl Loadable for AudioClip {
    fn insert_into(self, manager: &mut AssetManager) -> Asset<Self> {
        manager.add_audio_clip(self)
    }
}

type LoaderFn = Arc<dyn Fn(&[u8], &str, &mut AssetManager) -> Result<AnyHandle> + Send + Sync>;

struct LoaderRegistry {
    loaders: HashMap<String, LoaderFn>,
}

impl LoaderRegistry {
    fn new() -> Self {
        Self { loaders: HashMap::new() }
    }

    fn register<T: Loadable>(
        &mut self,
        ext: impl Into<String>,
        loader: impl Fn(&[u8], &str) -> Result<T> + Send + Sync + 'static,
    ) {
        let arc: LoaderFn = Arc::new(move |bytes, path, manager| {
            let value = loader(bytes, path)?;
            let handle = value.insert_into(manager);
            Ok(AnyHandle {
                index: handle.index(),
                generation: handle.generation(),
                type_id: TypeId::of::<T>(),
            })
        });
        self.loaders.insert(ext.into(), arc);
    }

    fn get(&self, ext: &str) -> Option<LoaderFn> {
        self.loaders.get(ext).cloned()
    }
}

pub struct AssetManager {
    images: AssetStore<Image>,
    fonts: AssetStore<Font>,
    texture_atlases: AssetStore<TextureAtlas>,
    audio_clips: AssetStore<AudioClip>,
    loader_registry: LoaderRegistry,
}

impl AssetManager {
    pub fn new() -> Self {
        let mut registry = LoaderRegistry::new();

        registry.register("png", |bytes, _| Image::from_bytes(bytes, false));
        registry.register("jpg", |bytes, _| Image::from_bytes(bytes, false));
        registry.register("jpeg", |bytes, _| Image::from_bytes(bytes, false));
        registry.register("ogg", |bytes, _| Ok(AudioClip::from_bytes(bytes.to_vec())));
        registry.register("wav", |bytes, _| Ok(AudioClip::from_bytes(bytes.to_vec())));
        registry.register("mp3", |bytes, _| Ok(AudioClip::from_bytes(bytes.to_vec())));

        Self {
            images: AssetStore::new(),
            fonts: AssetStore::new(),
            texture_atlases: AssetStore::new(),
            audio_clips: AssetStore::new(),
            loader_registry: registry,
        }
    }

    /// Register a loader for a file extension.
    pub fn register_loader<T: Loadable>(
        &mut self,
        ext: impl Into<String>,
        loader: impl Fn(&[u8], &str) -> Result<T> + Send + Sync + 'static,
    ) {
        self.loader_registry.register(ext, loader);
    }

    /// Load an asset from `path`, dispatching to the registered loader for its extension.
    pub fn load(&mut self, path: &str) -> Result<AnyHandle> {
        let resolved = resolve_asset_path(path);
        let bytes = std::fs::read(&resolved)
            .map_err(|e| anyhow!("Failed to read '{}': {}", path, e))?;
        let ext = resolved
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow!("Path '{}' has no file extension", path))?
            .to_string();
        // Clone Arc to end the borrow on loader_registry before passing &mut self
        let loader = self.loader_registry.get(&ext)
            .ok_or_else(|| anyhow!("No loader registered for extension '{}'", ext))?;
        loader(&bytes, path, self)
    }

    pub fn add_image(&mut self, image: Image) -> Asset<Image> {
        self.images.insert(image)
    }

    pub fn get_image(&self, handle: Asset<Image>) -> Option<&Image> {
        self.images.get(handle)
    }

    pub fn get_image_mut(&mut self, handle: Asset<Image>) -> Option<&mut Image> {
        self.images.get_mut(handle)
    }

    pub fn remove_image(&mut self, handle: Asset<Image>) -> Option<Image> {
        self.images.remove(handle)
    }

    pub fn add_font(&mut self, font: Font) -> Asset<Font> {
        self.fonts.insert(font)
    }

    pub fn get_font(&self, handle: Asset<Font>) -> Option<&Font> {
        self.fonts.get(handle)
    }

    pub fn get_font_mut(&mut self, handle: Asset<Font>) -> Option<&mut Font> {
        self.fonts.get_mut(handle)
    }

    pub fn remove_font(&mut self, handle: Asset<Font>) -> Option<Font> {
        self.fonts.remove(handle)
    }

    pub fn add_texture_atlas(&mut self, texture_atlas: TextureAtlas) -> Asset<TextureAtlas> {
        self.texture_atlases.insert(texture_atlas)
    }

    pub fn get_texture_atlas(&self, handle: Asset<TextureAtlas>) -> Option<&TextureAtlas> {
        self.texture_atlases.get(handle)
    }

    pub fn get_texture_atlas_mut(
        &mut self,
        handle: Asset<TextureAtlas>,
    ) -> Option<&mut TextureAtlas> {
        self.texture_atlases.get_mut(handle)
    }

    pub fn remove_texture_atlas(&mut self, handle: Asset<TextureAtlas>) -> Option<TextureAtlas> {
        self.texture_atlases.remove(handle)
    }

    pub fn add_audio_clip(&mut self, clip: AudioClip) -> Asset<AudioClip> {
        self.audio_clips.insert(clip)
    }

    pub fn get_audio_clip(&self, handle: Asset<AudioClip>) -> Option<&AudioClip> {
        self.audio_clips.get(handle)
    }

    pub fn get_audio_clip_mut(&mut self, handle: Asset<AudioClip>) -> Option<&mut AudioClip> {
        self.audio_clips.get_mut(handle)
    }

    pub fn remove_audio_clip(&mut self, handle: Asset<AudioClip>) -> Option<AudioClip> {
        self.audio_clips.remove(handle)
    }
}
