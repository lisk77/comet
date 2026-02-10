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
    swap_scratch: NonNull<u8>,
}

unsafe impl Send for Column {}
unsafe impl Sync for Column {}

impl Column {
    pub fn new<T: 'static>(capacity: usize) -> Self {
        let item_layout = Layout::new::<T>();
        let drop_fn = |ptr: *mut u8| unsafe { ptr::drop_in_place(ptr as *mut T) };
        let type_id = TypeId::of::<T>();

        if item_layout.size() == 0 {
            return Self {
                type_id,
                item_layout,
                drop_fn,
                data: NonNull::dangling(),
                len: 0,
                capacity: usize::MAX,
                swap_scratch: NonNull::dangling(),
            };
        }

        let swap_scratch =
            NonNull::new(unsafe { alloc(item_layout) }).unwrap_or_else(|| handle_alloc_error(item_layout));

        let mut column = Self {
            type_id,
            item_layout,
            drop_fn,
            data: NonNull::dangling(),
            len: 0,
            capacity: 0,
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
    fn assert_type<T: 'static>(&self) {
        assert_eq!(self.type_id, TypeId::of::<T>(), "Type mismatch");
    }

    fn reserve_exact(&mut self, additional: usize) {
        let available = self.capacity.saturating_sub(self.len);
        if available >= additional {
            return;
        }
        let increment = additional - available;
        self.grow_exact(increment);
    }

    fn grow_exact(&mut self, increment: usize) {
        debug_assert!(self.item_layout.size() != 0);
        let new_capacity = self.capacity + increment;
        let new_layout = array_layout(&self.item_layout, new_capacity)
            .expect("array layout should be valid");

        unsafe {
            let new_data = if self.capacity == 0 {
                alloc(new_layout)
            } else {
                realloc(
                    self.data.as_ptr(),
                    array_layout(&self.item_layout, self.capacity).expect("array layout should be valid"),
                    new_layout.size(),
                )
            };
            self.data = NonNull::new(new_data).unwrap_or_else(|| handle_alloc_error(new_layout));
        }
        self.capacity = new_capacity;
    }

    #[inline]
    unsafe fn elem_ptr(&self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len);
        self.data.as_ptr().add(index * self.item_layout.size())
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
        assert!(index1 < self.len && index2 < self.len, "Index out of bounds");
        if index1 == index2 || self.item_layout.size() == 0 {
            return;
        }
        unsafe {
            let ptr1 = self.elem_ptr(index1);
            let ptr2 = self.elem_ptr(index2);
            let size = self.item_layout.size();
            let scratch = self.swap_scratch.as_ptr();

            ptr::copy_nonoverlapping(ptr1, scratch, size);
            ptr::copy(ptr2, ptr1, size);
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
}

impl Drop for Column {
    fn drop(&mut self) {
        if self.item_layout.size() == 0 {
            return;
        }

        for i in 0..self.len {
            unsafe {
                let ptr = self.elem_ptr(i);
                (self.drop_fn)(ptr);
            }
        }

        if self.capacity == 0 {
            unsafe {
                dealloc(self.swap_scratch.as_ptr(), self.item_layout);
            }
            return;
        }

        let array_layout =
            array_layout(&self.item_layout, self.capacity).expect("array layout should be valid");
        unsafe {
            dealloc(self.data.as_ptr(), array_layout);
            dealloc(self.swap_scratch.as_ptr(), self.item_layout);
        }
    }
}

fn array_layout(layout: &Layout, n: usize) -> Option<Layout> {
    let (array_layout, offset) = repeat_layout(layout, n)?;
    debug_assert_eq!(layout.size(), offset);
    Some(array_layout)
}

fn repeat_layout(layout: &Layout, n: usize) -> Option<(Layout, usize)> {
    let padded_size = layout.size() + padding_needed_for(layout, layout.align());
    let alloc_size = padded_size.checked_mul(n)?;

    unsafe {
        Some((
            Layout::from_size_align_unchecked(alloc_size, layout.align()),
            padded_size,
        ))
    }
}

const fn padding_needed_for(layout: &Layout, align: usize) -> usize {
    let len = layout.size();
    let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
    len_rounded_up.wrapping_sub(len)
}
