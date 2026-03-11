use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::any::TypeId;
use std::mem;
use std::ptr::{self, NonNull};

pub struct Column {
    type_id: TypeId,
    item_layout: Layout,
    drop_fn: unsafe fn(*mut u8),

    data: NonNull<u8>,
    len: usize,
    capacity: usize,

    // Critical: stride (size padded up to alignment)
    stride: usize,

    swap_scratch: NonNull<u8>,
}

unsafe impl Send for Column {}
unsafe impl Sync for Column {}

impl Column {
    pub fn new_raw(
        type_id: TypeId,
        item_layout: Layout,
        drop_fn: unsafe fn(*mut u8),
        capacity: usize,
    ) -> Self {
        // ZST: no allocation, capacity effectively infinite
        if item_layout.size() == 0 {
            return Self {
                type_id,
                item_layout,
                drop_fn,
                data: NonNull::dangling(),
                len: 0,
                capacity: usize::MAX,
                stride: 0,
                swap_scratch: NonNull::dangling(),
            };
        }

        let stride = item_layout.pad_to_align().size();

        let swap_scratch = NonNull::new(unsafe { alloc(item_layout) })
            .unwrap_or_else(|| handle_alloc_error(item_layout));

        let mut column = Self {
            type_id,
            item_layout,
            drop_fn,
            data: NonNull::dangling(),
            len: 0,
            capacity: 0,
            stride,
            swap_scratch,
        };

        column.reserve_exact(capacity);
        column
    }

    pub fn new<T: 'static>(capacity: usize) -> Self {
        let item_layout = Layout::new::<T>();
        let drop_fn = |ptr: *mut u8| unsafe { ptr::drop_in_place(ptr as *mut T) };
        let type_id = TypeId::of::<T>();

        // ZST
        if item_layout.size() == 0 {
            return Self {
                type_id,
                item_layout,
                drop_fn,
                data: NonNull::dangling(),
                len: 0,
                capacity: usize::MAX,
                stride: 0,
                swap_scratch: NonNull::dangling(),
            };
        }

        let stride = item_layout.pad_to_align().size();

        let swap_scratch = NonNull::new(unsafe { alloc(item_layout) })
            .unwrap_or_else(|| handle_alloc_error(item_layout));

        let mut column = Self {
            type_id,
            item_layout,
            drop_fn,
            data: NonNull::dangling(),
            len: 0,
            capacity: 0,
            stride,
            swap_scratch,
        };

        column.reserve_exact(capacity);
        column
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    fn assert_type<T: 'static>(&self) {
        assert_eq!(self.type_id, TypeId::of::<T>(), "Type mismatch");
    }

    #[inline]
    fn assert_type_id(&self, type_id: TypeId) {
        assert_eq!(self.type_id, type_id, "Type mismatch");
    }

    fn reserve_exact(&mut self, additional: usize) {
        if self.item_layout.size() == 0 {
            return;
        }
        let required = self
            .len
            .checked_add(additional)
            .expect("column capacity overflow");

        if self.capacity >= required {
            return;
        }

        let mut new_capacity = self.capacity.max(4);
        while new_capacity < required {
            let doubled = new_capacity.saturating_mul(2);
            if doubled <= new_capacity {
                new_capacity = required;
                break;
            }
            new_capacity = doubled;
        }

        self.grow_to(new_capacity);
    }

    fn grow_to(&mut self, new_capacity: usize) {
        debug_assert!(self.item_layout.size() != 0);
        debug_assert!(new_capacity >= self.capacity);

        let new_layout = array_layout_strided(self.item_layout.align(), self.stride, new_capacity)
            .expect("array layout should be valid");

        unsafe {
            let new_data = if self.capacity == 0 {
                alloc(new_layout)
            } else {
                let old_layout =
                    array_layout_strided(self.item_layout.align(), self.stride, self.capacity)
                        .expect("array layout should be valid");
                realloc(self.data.as_ptr(), old_layout, new_layout.size())
            };

            self.data = NonNull::new(new_data).unwrap_or_else(|| handle_alloc_error(new_layout));
        }

        self.capacity = new_capacity;
    }

    #[inline]
    unsafe fn elem_ptr(&self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len);
        self.data.as_ptr().add(index * self.stride)
    }

    fn push_raw_from(&mut self, src: *const u8) {
        debug_assert!(self.item_layout.size() != 0);
        self.reserve_exact(1);

        let index = self.len;
        self.len += 1;

        unsafe {
            let dst_ptr = self.elem_ptr(index);
            ptr::copy_nonoverlapping(src, dst_ptr, self.item_layout.size());
        }
    }

    pub fn push<T: 'static>(&mut self, item: T) {
        self.assert_type::<T>();

        if self.item_layout.size() == 0 {
            self.len += 1;
            mem::forget(item);
            return;
        }

        self.reserve_exact(1);

        let index = self.len;
        self.len += 1;

        unsafe {
            let ptr = self.elem_ptr(index) as *mut T;
            ptr::write(ptr, item);
        }
    }

    /// # Safety
    /// Caller must guarantee that `T` matches this column's component type.
    pub unsafe fn push_unchecked<T: 'static>(&mut self, item: T) {
        if self.item_layout.size() == 0 {
            self.len += 1;
            mem::forget(item);
            return;
        }

        self.reserve_exact(1);

        let index = self.len;
        self.len += 1;

        unsafe {
            let ptr = self.elem_ptr(index) as *mut T;
            ptr::write(ptr, item);
        }
    }

    pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
        self.assert_type::<T>();
        if index >= self.len {
            return None;
        }
        unsafe { Some(&*(self.elem_ptr(index) as *const T)) }
    }

    pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
        self.assert_type::<T>();
        if index >= self.len {
            return None;
        }
        unsafe { Some(&mut *(self.elem_ptr(index) as *mut T)) }
    }

    pub fn set<T: 'static>(&mut self, index: usize, item: T) -> Option<()> {
        self.assert_type::<T>();
        if index >= self.len {
            return None;
        }
        unsafe {
            let ptr = self.elem_ptr(index) as *mut T;
            ptr::drop_in_place(ptr);
            ptr::write(ptr, item);
        }
        Some(())
    }

    pub fn remove<T: 'static>(&mut self, index: usize) -> Option<T> {
        self.assert_type::<T>();
        if index >= self.len {
            return None;
        }
        unsafe { Some(self.swap_remove_unchecked::<T>(index)) }
    }

    pub fn remove_any(&mut self, index: usize) {
        if index >= self.len {
            return;
        }
        unsafe {
            let ptr = self.elem_ptr(index);
            (self.drop_fn)(ptr);
            self.swap_last_into(index);
        }
    }

    pub fn swap(&mut self, index1: usize, index2: usize) {
        assert!(
            index1 < self.len && index2 < self.len,
            "Index out of bounds"
        );

        if index1 == index2 || self.item_layout.size() == 0 {
            return;
        }

        unsafe {
            let ptr1 = self.elem_ptr(index1);
            let ptr2 = self.elem_ptr(index2);
            let size = self.item_layout.size();
            let scratch = self.swap_scratch.as_ptr();

            ptr::copy_nonoverlapping(ptr1, scratch, size);
            ptr::copy_nonoverlapping(ptr2, ptr1, size);
            ptr::copy_nonoverlapping(scratch, ptr2, size);
        }
    }

    unsafe fn swap_remove_unchecked<T: 'static>(&mut self, index: usize) -> T {
        if self.item_layout.size() == 0 {
            self.len -= 1;
            return mem::zeroed();
        }

        let ptr = self.elem_ptr(index) as *mut T;
        let value = ptr::read(ptr);
        self.swap_last_into(index);
        value
    }

    unsafe fn swap_last_into(&mut self, index: usize) {
        let last = self.len - 1;
        if index != last {
            let last_ptr = self.elem_ptr(last);
            let dst_ptr = self.elem_ptr(index);
            ptr::copy_nonoverlapping(last_ptr, dst_ptr, self.item_layout.size());
        }
        self.len -= 1;
    }

    pub fn move_last_to(&mut self, dst: &mut Column) -> Option<()> {
        self.assert_type_id(dst.type_id);

        if self.len == 0 {
            return None;
        }

        if self.item_layout.size() == 0 {
            self.len -= 1;
            dst.len += 1;
            return Some(());
        }

        let src_index = self.len - 1;
        let src_ptr = unsafe { self.elem_ptr(src_index) as *const u8 };

        dst.push_raw_from(src_ptr);

        // Treat the source slot as moved-from by reducing len.
        self.len -= 1;

        Some(())
    }

    pub fn drop_last(&mut self) -> Option<()> {
        if self.len == 0 {
            return None;
        }

        if self.item_layout.size() == 0 {
            unsafe {
                (self.drop_fn)(NonNull::<u8>::dangling().as_ptr());
            }
            self.len -= 1;
            return Some(());
        }

        let index = self.len - 1;
        unsafe {
            let ptr = self.elem_ptr(index);
            (self.drop_fn)(ptr);
        }

        self.len -= 1;
        Some(())
    }
}

impl Drop for Column {
    fn drop(&mut self) {
        if self.item_layout.size() == 0 {
            for _ in 0..self.len {
                unsafe {
                    (self.drop_fn)(NonNull::<u8>::dangling().as_ptr());
                }
            }
            return;
        }

        for i in 0..self.len {
            unsafe {
                let ptr = self.elem_ptr(i);
                (self.drop_fn)(ptr);
            }
        }

        unsafe {
            dealloc(self.swap_scratch.as_ptr(), self.item_layout);
        }

        if self.capacity == 0 {
            return;
        }

        let array_layout =
            array_layout_strided(self.item_layout.align(), self.stride, self.capacity)
                .expect("array layout should be valid");

        unsafe {
            dealloc(self.data.as_ptr(), array_layout);
        }
    }
}

fn array_layout_strided(align: usize, stride: usize, n: usize) -> Option<Layout> {
    let alloc_size = stride.checked_mul(n)?;
    Layout::from_size_align(alloc_size, align).ok()
}

#[cfg(test)]
mod tests {
    use super::Column;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DROPS: AtomicUsize = AtomicUsize::new(0);

    struct ZstDrop;

    impl Drop for ZstDrop {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn zst_components_are_dropped() {
        DROPS.store(0, Ordering::SeqCst);

        {
            let mut column = Column::new::<ZstDrop>(0);
            column.push(ZstDrop);
            column.push(ZstDrop);
            let _ = column.drop_last();
        }

        assert_eq!(DROPS.load(Ordering::SeqCst), 2);
    }
}
