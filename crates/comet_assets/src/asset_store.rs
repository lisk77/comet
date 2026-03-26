use std::collections::HashMap;
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

struct OwnedData {
    ptr: *mut u8,
    drop_fn: unsafe fn(*mut u8),
}

unsafe impl Send for OwnedData {}
unsafe impl Sync for OwnedData {}

impl OwnedData {
    fn new<T>(value: T) -> Self {
        unsafe fn drop_impl<T>(ptr: *mut u8) {
            drop(Box::from_raw(ptr as *mut T));
        }
        Self {
            ptr: Box::into_raw(Box::new(value)) as *mut u8,
            drop_fn: drop_impl::<T>,
        }
    }

    unsafe fn as_ref<T>(&self) -> &T {
        &*(self.ptr as *const T)
    }

    unsafe fn as_mut<T>(&mut self) -> &mut T {
        &mut *(self.ptr as *mut T)
    }

    unsafe fn take<T>(self) -> T {
        let val = *Box::from_raw(self.ptr as *mut T);
        std::mem::forget(self);
        val
    }
}

impl Drop for OwnedData {
    fn drop(&mut self) {
        unsafe { (self.drop_fn)(self.ptr) }
    }
}

enum TryRecvResult {
    Ready(OwnedData),
    Empty,
    Failed,
}

enum RecvResult {
    Ready(OwnedData),
    Failed,
}

struct PendingData {
    receiver: *mut u8,
    try_recv_fn: unsafe fn(*mut u8) -> TryRecvResult,
    recv_fn: unsafe fn(*mut u8) -> RecvResult,
    drop_fn: unsafe fn(*mut u8),
}

unsafe impl Send for PendingData {}
unsafe impl Sync for PendingData {}

unsafe fn try_recv_impl<T: 'static>(ptr: *mut u8) -> TryRecvResult {
    let rx = &*(ptr as *const mpsc::Receiver<Result<T>>);
    match rx.try_recv() {
        Ok(Ok(value)) => TryRecvResult::Ready(OwnedData::new(value)),
        Ok(Err(e)) => { error!("Asset load failed: {}", e); TryRecvResult::Failed }
        Err(mpsc::TryRecvError::Disconnected) => { error!("Asset load channel closed unexpectedly"); TryRecvResult::Failed }
        Err(mpsc::TryRecvError::Empty) => TryRecvResult::Empty,
    }
}

unsafe fn recv_impl<T: 'static>(ptr: *mut u8) -> RecvResult {
    let rx = *Box::from_raw(ptr as *mut mpsc::Receiver<Result<T>>);
    match rx.recv() {
        Ok(Ok(value)) => RecvResult::Ready(OwnedData::new(value)),
        Ok(Err(e)) => { error!("Asset load failed: {}", e); RecvResult::Failed }
        Err(_) => { error!("Asset load channel closed unexpectedly"); RecvResult::Failed }
    }
}

unsafe fn drop_receiver_impl<T: 'static>(ptr: *mut u8) {
    drop(Box::from_raw(ptr as *mut mpsc::Receiver<Result<T>>));
}

impl PendingData {
    fn new<T: Send + 'static>(rx: mpsc::Receiver<Result<T>>) -> Self {
        Self {
            receiver: Box::into_raw(Box::new(rx)) as *mut u8,
            try_recv_fn: try_recv_impl::<T>,
            recv_fn: recv_impl::<T>,
            drop_fn: drop_receiver_impl::<T>,
        }
    }

    fn try_recv(&self) -> TryRecvResult {
        unsafe { (self.try_recv_fn)(self.receiver) }
    }

    fn recv_blocking(self) -> RecvResult {
        let result = unsafe { (self.recv_fn)(self.receiver) };
        std::mem::forget(self);
        result
    }
}

impl Drop for PendingData {
    fn drop(&mut self) {
        unsafe { (self.drop_fn)(self.receiver) }
    }
}

enum SlotState {
    Ready(OwnedData),
    Pending(PendingData),
    Failed,
}

struct Slot {
    generation: u32,
    value: Option<SlotState>,
}

pub struct AssetStore {
    slots: Vec<Slot>,
    free_list: Vec<u32>,
    paths: HashMap<String, (u32, u32)>,
}

unsafe impl Send for AssetStore {}
unsafe impl Sync for AssetStore {}

impl AssetStore {
    pub fn new() -> Self {
        Self { slots: Vec::new(), free_list: Vec::new(), paths: HashMap::new() }
    }

    fn alloc_slot(&mut self, state: SlotState) -> (u32, u32) {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index as usize];
            debug_assert!(slot.value.is_none());
            slot.value = Some(state);
            (index, slot.generation)
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot { generation: 0, value: Some(state) });
            (index, 0)
        }
    }

    fn resolve_pending(slot: &mut Slot) -> bool {
        if !matches!(&slot.value, Some(SlotState::Pending(_))) { return true; }
        if let Some(SlotState::Pending(pending)) = slot.value.take() {
            match pending.recv_blocking() {
                RecvResult::Ready(data) => { slot.value = Some(SlotState::Ready(data)); true }
                RecvResult::Failed => { slot.value = Some(SlotState::Failed); false }
            }
        } else {
            true
        }
    }

    pub fn insert<T: 'static>(&mut self, asset: T) -> Asset<T> {
        let (index, generation) = self.alloc_slot(SlotState::Ready(OwnedData::new(asset)));
        Asset::new(index, generation)
    }

    pub(crate) fn insert_pending<T: Send + 'static>(&mut self) -> (Asset<T>, mpsc::Sender<Result<T>>) {
        let (tx, rx) = mpsc::channel::<Result<T>>();
        let (index, generation) = self.alloc_slot(SlotState::Pending(PendingData::new(rx)));
        (Asset::new(index, generation), tx)
    }

    pub fn load_state<T: 'static>(&mut self, handle: Asset<T>) -> LoadState {
        let index = handle.index() as usize;
        let Some(slot) = self.slots.get_mut(index) else { return LoadState::Failed; };
        if slot.generation != handle.generation() { return LoadState::Failed; }

        match slot.value.take() {
            None | Some(SlotState::Failed) => {
                slot.value = Some(SlotState::Failed);
                LoadState::Failed
            }
            Some(SlotState::Ready(data)) => {
                slot.value = Some(SlotState::Ready(data));
                LoadState::Ready
            }
            Some(SlotState::Pending(pending)) => match pending.try_recv() {
                TryRecvResult::Ready(data) => {
                    slot.value = Some(SlotState::Ready(data));
                    LoadState::Ready
                }
                TryRecvResult::Failed => {
                    slot.value = Some(SlotState::Failed);
                    LoadState::Failed
                }
                TryRecvResult::Empty => {
                    slot.value = Some(SlotState::Pending(pending));
                    LoadState::Loading
                }
            },
        }
    }

    pub fn is_ready<T: 'static>(&mut self, handle: Asset<T>) -> bool {
        matches!(self.load_state(handle), LoadState::Ready)
    }

    pub fn get<T: 'static>(&mut self, handle: Asset<T>) -> Option<&T> {
        let slot = self.slots.get_mut(handle.index() as usize)?;
        if slot.generation != handle.generation() { return None; }
        if !Self::resolve_pending(slot) { return None; }
        match slot.value.as_ref()? {
            SlotState::Ready(data) => Some(unsafe { data.as_ref::<T>() }),
            _ => None,
        }
    }

    pub fn get_mut<T: 'static>(&mut self, handle: Asset<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.index() as usize)?;
        if slot.generation != handle.generation() { return None; }
        if !Self::resolve_pending(slot) { return None; }
        match slot.value.as_mut()? {
            SlotState::Ready(data) => Some(unsafe { data.as_mut::<T>() }),
            _ => None,
        }
    }

    pub fn remove<T: 'static>(&mut self, handle: Asset<T>) -> Option<T> {
        let index = handle.index() as usize;
        let slot = self.slots.get_mut(index)?;
        if slot.generation != handle.generation() { return None; }

        if matches!(&slot.value, Some(SlotState::Pending(_))) {
            if let Some(SlotState::Pending(pending)) = slot.value.take() {
                match pending.recv_blocking() {
                    RecvResult::Ready(data) => slot.value = Some(SlotState::Ready(data)),
                    RecvResult::Failed => {
                        slot.generation = slot.generation.wrapping_add(1);
                        self.free_list.push(handle.index());
                        return None;
                    }
                }
            }
        }

        match slot.value.take()? {
            SlotState::Ready(data) => {
                slot.generation = slot.generation.wrapping_add(1);
                self.free_list.push(handle.index());
                Some(unsafe { data.take::<T>() })
            }
            _ => {
                slot.generation = slot.generation.wrapping_add(1);
                self.free_list.push(handle.index());
                None
            }
        }
    }

    pub(crate) fn record_path(&mut self, index: u32, generation: u32, stem: &str) {
        self.paths.insert(stem.to_string(), (index, generation));
    }

    pub fn find_by_stem<T: 'static>(&self, stem: &str) -> Option<Asset<T>> {
        self.paths.get(stem).map(|&(index, gen)| Asset::new(index, gen))
    }

    pub fn contains<T: 'static>(&self, handle: Asset<T>) -> bool {
        let Some(slot) = self.slots.get(handle.index() as usize) else { return false; };
        slot.generation == handle.generation() && slot.value.is_some()
    }
}
