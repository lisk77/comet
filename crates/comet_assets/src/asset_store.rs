use crate::asset_handle::Asset;

struct Slot<T> {
    generation: u32,
    value: Option<T>
}

pub struct AssetStore<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<u32>,
}

impl<T> AssetStore<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn insert(&mut self, asset: T) -> Asset<T> {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index as usize];
            debug_assert!(slot.value.is_none());

            slot.value = Some(asset);
            Asset::new(index, slot.generation)
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot {
                generation: 0,
                value: Some(asset),
            });
            Asset::new(index, 0)
        }
    }

    pub fn get(&self, handle: Asset<T>) -> Option<&T> {
        let slot = self.slots.get(handle.index() as usize)?;
        if slot.generation != handle.generation() {
            return None;
        }
        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, handle: Asset<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.index() as usize)?;
        if slot.generation != handle.generation() {
            return None;
        }
        slot.value.as_mut()
    }

    pub fn remove(&mut self, handle: Asset<T>) -> Option<T> {
        let slot = self.slots.get_mut(handle.index() as usize)?;
        if slot.generation != handle.generation() {
            return None;
        }

        let value = slot.value.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free_list.push(handle.index());
        Some(value)
    }

    pub fn contains(&self, handle: Asset<T>) -> bool {
        self.get(handle).is_some()
    }
}
