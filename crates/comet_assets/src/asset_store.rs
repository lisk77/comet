use std::sync::mpsc;
use crate::asset_handle::Asset;
use anyhow::Result;
use comet_log::error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    Loading,
    Ready,
    Failed,
}

enum SlotState<T> {
    Ready(T),
    Pending(mpsc::Receiver<Result<T>>),
    Failed,
}

struct Slot<T> {
    generation: u32,
    value: Option<SlotState<T>>,
}

pub struct AssetStore<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<u32>,
}

unsafe impl<T: Send> Sync for AssetStore<T> {}

impl<T> AssetStore<T> {
    pub fn new() -> Self {
        Self { slots: Vec::new(), free_list: Vec::new() }
    }

    fn alloc_slot(&mut self, state: SlotState<T>) -> Asset<T> {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index as usize];
            debug_assert!(slot.value.is_none());
            slot.value = Some(state);
            Asset::new(index, slot.generation)
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot { generation: 0, value: Some(state) });
            Asset::new(index, 0)
        }
    }

    /// Blocks until a Pending slot resolves. Sets Failed on error. Returns false if failed.
    fn resolve_pending(slot: &mut Slot<T>) -> bool {
        if !matches!(&slot.value, Some(SlotState::Pending(_))) { return true; }
        if let Some(SlotState::Pending(rx)) = slot.value.take() {
            match rx.recv() {
                Ok(Ok(value)) => { slot.value = Some(SlotState::Ready(value)); true }
                Ok(Err(e)) => { error!("Asset load failed: {}", e); slot.value = Some(SlotState::Failed); false }
                Err(_) => { error!("Asset load channel closed unexpectedly"); slot.value = Some(SlotState::Failed); false }
            }
        } else {
            true
        }
    }

    pub fn insert(&mut self, asset: T) -> Asset<T> {
        self.alloc_slot(SlotState::Ready(asset))
    }

    /// Pre-allocate a Pending slot. The background thread sends its result via the returned Sender.
    pub(crate) fn insert_pending(&mut self) -> (Asset<T>, mpsc::Sender<Result<T>>) {
        let (tx, rx) = mpsc::channel();
        (self.alloc_slot(SlotState::Pending(rx)), tx)
    }

    /// Non-blocking load state check. Transitions Pending → Ready/Failed via try_recv.
    pub fn load_state(&mut self, handle: Asset<T>) -> LoadState {
        let index = handle.index() as usize;
        let Some(slot) = self.slots.get_mut(index) else { return LoadState::Failed; };
        if slot.generation != handle.generation() { return LoadState::Failed; }

        match slot.value.take() {
            None | Some(SlotState::Failed) => {
                slot.value = Some(SlotState::Failed);
                LoadState::Failed
            }
            Some(SlotState::Ready(v)) => {
                slot.value = Some(SlotState::Ready(v));
                LoadState::Ready
            }
            Some(SlotState::Pending(rx)) => match rx.try_recv() {
                Ok(Ok(value)) => {
                    slot.value = Some(SlotState::Ready(value));
                    LoadState::Ready
                }
                Ok(Err(e)) => {
                    error!("Asset load failed: {}", e);
                    slot.value = Some(SlotState::Failed);
                    LoadState::Failed
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    error!("Asset load channel closed unexpectedly");
                    slot.value = Some(SlotState::Failed);
                    LoadState::Failed
                }
                Err(mpsc::TryRecvError::Empty) => {
                    slot.value = Some(SlotState::Pending(rx));
                    LoadState::Loading
                }
            },
        }
    }

    pub fn is_ready(&mut self, handle: Asset<T>) -> bool {
        matches!(self.load_state(handle), LoadState::Ready)
    }

    /// Blocking get — resolves Pending by waiting for the channel.
    pub fn get(&mut self, handle: Asset<T>) -> Option<&T> {
        let slot = self.slots.get_mut(handle.index() as usize)?;
        if slot.generation != handle.generation() { return None; }
        if !Self::resolve_pending(slot) { return None; }
        match slot.value.as_ref()? {
            SlotState::Ready(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, handle: Asset<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.index() as usize)?;
        if slot.generation != handle.generation() { return None; }
        if !Self::resolve_pending(slot) { return None; }
        match slot.value.as_mut()? {
            SlotState::Ready(v) => Some(v),
            _ => None,
        }
    }

    pub fn remove(&mut self, handle: Asset<T>) -> Option<T> {
        let index = handle.index() as usize;
        let slot = self.slots.get_mut(index)?;
        if slot.generation != handle.generation() { return None; }

        if matches!(&slot.value, Some(SlotState::Pending(_))) {
            if let Some(SlotState::Pending(rx)) = slot.value.take() {
                match rx.recv() {
                    Ok(Ok(value)) => slot.value = Some(SlotState::Ready(value)),
                    Ok(Err(_)) | Err(_) => {
                        slot.generation = slot.generation.wrapping_add(1);
                        self.free_list.push(handle.index());
                        return None;
                    }
                }
            }
        }

        match slot.value.take()? {
            SlotState::Ready(value) => {
                slot.generation = slot.generation.wrapping_add(1);
                self.free_list.push(handle.index());
                Some(value)
            }
            _ => {
                slot.generation = slot.generation.wrapping_add(1);
                self.free_list.push(handle.index());
                None
            }
        }
    }

    pub fn contains(&self, handle: Asset<T>) -> bool {
        let Some(slot) = self.slots.get(handle.index() as usize) else { return false; };
        slot.generation == handle.generation() && slot.value.is_some()
    }
}
