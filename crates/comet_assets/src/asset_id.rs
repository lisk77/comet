use crate::Asset;
use std::any::TypeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AssetId {
    asset_type: TypeId,
    index: u32,
    generation: u32,
}

impl AssetId {
    pub(crate) fn new<T: 'static>(index: u32, generation: u32) -> Self {
        Self {
            asset_type: TypeId::of::<T>(),
            index,
            generation,
        }
    }

    pub fn asset_type(&self) -> TypeId {
        self.asset_type
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn is<T: 'static>(&self) -> bool {
        self.asset_type == TypeId::of::<T>()
    }

    pub fn typed<T: 'static>(self) -> Option<Asset<T>> {
        self.is::<T>()
            .then(|| Asset::new(self.index, self.generation))
    }
}
