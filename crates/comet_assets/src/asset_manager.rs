use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use crate::{asset_store::*, asset_handle::*, image::Image, font::Font, texture_atlas::TextureAtlas, audio_clip::AudioClip};

/// Trait for asset types that can be inserted into the `AssetManager`.
pub trait Loadable: Send + Sync + 'static {
    fn insert_into(self, manager: &mut AssetManager) -> Asset<Self>
    where
        Self: Sized;

    fn insert_pending(manager: &mut AssetManager) -> (Asset<Self>, std::sync::mpsc::Sender<anyhow::Result<Self>>)
    where
        Self: Sized;

    fn load_state(handle: Asset<Self>, manager: &mut AssetManager) -> LoadState
    where
        Self: Sized;
}

impl Loadable for Image {
    fn insert_into(self, manager: &mut AssetManager) -> Asset<Self> { manager.add_image(self) }
    fn insert_pending(manager: &mut AssetManager) -> (Asset<Self>, std::sync::mpsc::Sender<anyhow::Result<Self>>) {
        manager.images.insert_pending()
    }
    fn load_state(handle: Asset<Self>, manager: &mut AssetManager) -> LoadState {
        manager.images.load_state(handle)
    }
}

impl Loadable for Font {
    fn insert_into(self, manager: &mut AssetManager) -> Asset<Self> { manager.add_font(self) }
    fn insert_pending(manager: &mut AssetManager) -> (Asset<Self>, std::sync::mpsc::Sender<anyhow::Result<Self>>) {
        manager.fonts.insert_pending()
    }
    fn load_state(handle: Asset<Self>, manager: &mut AssetManager) -> LoadState {
        manager.fonts.load_state(handle)
    }
}

impl Loadable for TextureAtlas {
    fn insert_into(self, manager: &mut AssetManager) -> Asset<Self> { manager.add_texture_atlas(self) }
    fn insert_pending(manager: &mut AssetManager) -> (Asset<Self>, std::sync::mpsc::Sender<anyhow::Result<Self>>) {
        manager.texture_atlases.insert_pending()
    }
    fn load_state(handle: Asset<Self>, manager: &mut AssetManager) -> LoadState {
        manager.texture_atlases.load_state(handle)
    }
}

impl Loadable for AudioClip {
    fn insert_into(self, manager: &mut AssetManager) -> Asset<Self> { manager.add_audio_clip(self) }
    fn insert_pending(manager: &mut AssetManager) -> (Asset<Self>, std::sync::mpsc::Sender<anyhow::Result<Self>>) {
        manager.audio_clips.insert_pending()
    }
    fn load_state(handle: Asset<Self>, manager: &mut AssetManager) -> LoadState {
        manager.audio_clips.load_state(handle)
    }
}

pub(crate) type AllocFn = Arc<dyn Fn(&mut AssetManager) -> (u32, u32, Box<dyn FnOnce(Vec<u8>, String) + Send>) + Send + Sync>;

struct LoaderEntry {
    type_id: TypeId,
    alloc: AllocFn,
}

struct LoaderRegistry {
    loaders: HashMap<String, LoaderEntry>,
}

impl LoaderRegistry {
    fn new() -> Self { Self { loaders: HashMap::new() } }

    fn register<T: Loadable>(
        &mut self,
        ext: impl Into<String>,
        loader: impl Fn(&[u8], &str) -> Result<T> + Send + Sync + 'static,
    ) {
        let loader = Arc::new(loader);

        let alloc: AllocFn = Arc::new(move |manager: &mut AssetManager| {
            let (handle, tx) = T::insert_pending(manager);
            let l = loader.clone();
            let worker: Box<dyn FnOnce(Vec<u8>, String) + Send> = Box::new(move |bytes: Vec<u8>, path: String| {
                let _ = tx.send(l(&bytes, &path));
            });
            (handle.index(), handle.generation(), worker)
        });

        self.loaders.insert(ext.into(), LoaderEntry { type_id: TypeId::of::<T>(), alloc });
    }

    fn get_alloc_typed<T: 'static>(&self, ext: &str) -> Option<AllocFn> {
        self.loaders.get(ext)
            .filter(|e| e.type_id == TypeId::of::<T>())
            .map(|e| e.alloc.clone())
    }
}

pub struct AssetManager {
    pub(crate) images: AssetStore<Image>,
    pub(crate) fonts: AssetStore<Font>,
    pub(crate) texture_atlases: AssetStore<TextureAtlas>,
    pub(crate) audio_clips: AssetStore<AudioClip>,
    loader_registry: LoaderRegistry,
}

impl AssetManager {
    pub fn new() -> Self {
        let mut registry = LoaderRegistry::new();

        registry.register("png", |bytes, _| Image::from_bytes(bytes, false));
        registry.register("jpg", |bytes, _| Image::from_bytes(bytes, false));
        registry.register("jpeg", |bytes, _| Image::from_bytes(bytes, false));
        registry.register("ttf", |bytes, path| Ok(Font::from_raw(bytes.to_vec(), path.to_string())));
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

    pub(crate) fn get_alloc_loader_typed<T: Loadable>(&self, ext: &str) -> Option<AllocFn> {
        self.loader_registry.get_alloc_typed::<T>(ext)
    }

    pub fn add_image(&mut self, image: Image) -> Asset<Image> {
        self.images.insert(image)
    }

    pub fn get_image(&mut self, handle: Asset<Image>) -> Option<&Image> {
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

    pub fn get_font(&mut self, handle: Asset<Font>) -> Option<&Font> {
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

    pub fn get_texture_atlas(&mut self, handle: Asset<TextureAtlas>) -> Option<&TextureAtlas> {
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

    pub fn get_audio_clip(&mut self, handle: Asset<AudioClip>) -> Option<&AudioClip> {
        self.audio_clips.get(handle)
    }

    pub fn get_audio_clip_mut(&mut self, handle: Asset<AudioClip>) -> Option<&mut AudioClip> {
        self.audio_clips.get_mut(handle)
    }

    pub fn remove_audio_clip(&mut self, handle: Asset<AudioClip>) -> Option<AudioClip> {
        self.audio_clips.remove(handle)
    }
}
