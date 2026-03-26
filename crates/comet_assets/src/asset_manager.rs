use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use crate::{asset_store::*, asset_handle::*, image::Image, font::Font, texture_atlas::TextureAtlas, audio_clip::AudioClip};

pub trait Loadable: Send + Sync + 'static {}

impl Loadable for Image {}
impl Loadable for Font {}
impl Loadable for TextureAtlas {}
impl Loadable for AudioClip {}

pub(crate) type AllocFn = Arc<dyn Fn(&mut AssetManager) -> (u32, u32, Box<dyn FnOnce(Vec<u8>, String) + Send>) + Send + Sync>;
pub(crate) type ReloadFn = Arc<dyn Fn(&mut AssetManager, u32) -> Option<Box<dyn FnOnce(Vec<u8>, String) + Send>> + Send + Sync>;

struct LoaderEntry {
    type_id: TypeId,
    alloc: AllocFn,
    reload: ReloadFn,
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

        let alloc_loader = Arc::clone(&loader);
        let alloc: AllocFn = Arc::new(move |manager: &mut AssetManager| {
            let (handle, tx) = manager.stores.get_mut::<T>().insert_pending::<T>();
            let l = Arc::clone(&alloc_loader);
            let worker: Box<dyn FnOnce(Vec<u8>, String) + Send> = Box::new(move |bytes: Vec<u8>, path: String| {
                let _ = tx.send(l(&bytes, &path));
            });
            (handle.index(), handle.generation(), worker)
        });

        let reload_loader = Arc::clone(&loader);
        let reload: ReloadFn = Arc::new(move |manager: &mut AssetManager, index: u32| {
            let tx = manager.stores.get_mut::<T>().set_reload_pending::<T>(index)?;
            let l = Arc::clone(&reload_loader);
            Some(Box::new(move |bytes: Vec<u8>, path: String| {
                let _ = tx.send(l(&bytes, &path));
            }) as Box<dyn FnOnce(Vec<u8>, String) + Send>)
        });

        self.loaders.insert(ext.into(), LoaderEntry { type_id: TypeId::of::<T>(), alloc, reload });
    }

    fn get_alloc_typed<T: 'static>(&self, ext: &str) -> Option<AllocFn> {
        self.loaders.get(ext)
            .filter(|e| e.type_id == TypeId::of::<T>())
            .map(|e| e.alloc.clone())
    }

    fn get_reload(&self, ext: &str, type_id: TypeId) -> Option<ReloadFn> {
        self.loaders.get(ext)
            .filter(|e| e.type_id == type_id)
            .map(|e| e.reload.clone())
    }
}

struct StoreMap {
    map: HashMap<TypeId, AssetStore>,
}

impl StoreMap {
    fn new() -> Self { Self { map: HashMap::new() } }

    fn register<T: Loadable>(&mut self) {
        self.map.entry(TypeId::of::<T>())
            .or_insert_with(AssetStore::new);
    }

    fn get_mut<T: Loadable>(&mut self) -> &mut AssetStore {
        self.map.get_mut(&TypeId::of::<T>())
            .expect("asset store not registered")
    }

    fn get<T: Loadable>(&self) -> Option<&AssetStore> {
        self.map.get(&TypeId::of::<T>())
    }
}

pub struct AssetManager {
    stores: StoreMap,
    loader_registry: LoaderRegistry,
}

impl AssetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            stores: StoreMap::new(),
            loader_registry: LoaderRegistry::new(),
        };

        manager.register_loader("png", |bytes, _| Image::from_bytes(bytes, false));
        manager.register_loader("jpg", |bytes, _| Image::from_bytes(bytes, false));
        manager.register_loader("jpeg", |bytes, _| Image::from_bytes(bytes, false));
        manager.register_loader("ttf", |bytes, path| Ok(Font::from_raw(bytes.to_vec(), path.to_string())));
        manager.register_loader("ogg", |bytes, _| Ok(AudioClip::from_bytes(bytes.to_vec())));
        manager.register_loader("wav", |bytes, _| Ok(AudioClip::from_bytes(bytes.to_vec())));
        manager.register_loader("mp3", |bytes, _| Ok(AudioClip::from_bytes(bytes.to_vec())));

        manager.stores.register::<TextureAtlas>();

        manager
    }

    pub fn register_asset_type<T: Loadable>(&mut self) {
        self.stores.register::<T>();
    }

    pub fn register_loader<T: Loadable>(
        &mut self,
        ext: impl Into<String>,
        loader: impl Fn(&[u8], &str) -> Result<T> + Send + Sync + 'static,
    ) {
        self.stores.register::<T>();
        self.loader_registry.register(ext, loader);
    }

    pub(crate) fn get_alloc_loader_typed<T: Loadable>(&self, ext: &str) -> Option<AllocFn> {
        self.loader_registry.get_alloc_typed::<T>(ext)
    }

    pub(crate) fn begin_reload(&mut self, ext: &str, type_id: TypeId, index: u32) -> Option<Box<dyn FnOnce(Vec<u8>, String) + Send>> {
        let reload = self.loader_registry.get_reload(ext, type_id)?;
        reload(self, index)
    }

    pub fn add<T: Loadable>(&mut self, asset: T) -> Asset<T> {
        self.stores.get_mut::<T>().insert(asset)
    }

    pub fn get<T: Loadable>(&mut self, handle: Asset<T>) -> Option<&T> {
        self.stores.get_mut::<T>().get(handle)
    }

    pub fn get_mut<T: Loadable>(&mut self, handle: Asset<T>) -> Option<&mut T> {
        self.stores.get_mut::<T>().get_mut(handle)
    }

    pub fn remove<T: Loadable>(&mut self, handle: Asset<T>) -> Option<T> {
        self.stores.get_mut::<T>().remove(handle)
    }

    pub fn load_state<T: Loadable>(&mut self, handle: Asset<T>) -> LoadState {
        self.stores.get_mut::<T>().load_state(handle)
    }

    pub(crate) fn record_path<T: Loadable>(&mut self, index: u32, generation: u32, path: &str) {
        self.stores.get_mut::<T>().record_path(index, generation, path);
    }

    pub fn find_by_path<T: Loadable>(&self, path: &str) -> Option<Asset<T>> {
        self.stores.get::<T>()?.find_by_path::<T>(path)
    }

    pub fn find_by_stem<T: Loadable>(&self, stem: &str) -> Option<Asset<T>> {
        self.stores.get::<T>()?.find_by_stem::<T>(stem)
    }
}
