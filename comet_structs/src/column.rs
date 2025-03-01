use crate::{Component};
use std::{
	alloc::{
		handle_alloc_error,
		Layout
	},
	any::TypeId,
	collections::{
		HashMap,
		HashSet
	},
	hash::{
		DefaultHasher,
		Hash,
		Hasher
	},
	mem::MaybeUninit,
	ptr::NonNull
};
use std::ptr;

#[derive(Debug, Clone)]
pub struct BlobVec {
	item_layout: Layout,
	capacity: usize,
	len: usize,
	data: NonNull<u8>,
	swap_scratch: NonNull<u8>,
	drop: unsafe fn(*mut u8)
}


impl BlobVec {
	pub fn new(item_layout: Layout, drop: unsafe fn(*mut u8), capacity: usize) -> Self {
		if item_layout.size() == 0 {
			BlobVec {
				swap_scratch: NonNull::dangling(),
				data: NonNull::dangling(),
				capacity: usize:: MAX,
				len: 0,
				item_layout,
				drop,
			}
		}
		else {
			let swap_scratch = NonNull::new(unsafe { std::alloc::alloc(item_layout) })
				.unwrap_or_else(|| handle_alloc_error(item_layout));

			let mut blob_vec = BlobVec {
				swap_scratch,
				data: NonNull::dangling(),
				capacity: 0,
				len: 0,
				item_layout,
				drop,
			};
			blob_vec.reserve_exact(capacity);
			blob_vec
		}
	}

	pub fn reserve_exact(&mut self, additional: usize) {
		let available_space = self.capacity - self.len;
		if available_space < additional {
			self.grow_exact(additional - available_space);
		}
	}

	fn grow_exact(&mut self, increment: usize) {
		debug_assert!(self.item_layout.size() != 0);

		let new_capacity = self.capacity + increment;
		let new_layout =
			array_layout(&self.item_layout, new_capacity).expect("array layout should be valid");
		unsafe {
			let new_data = if self.capacity == 0 {
				std::alloc::alloc(new_layout)
			} else {
				std::alloc::realloc(
					self.get_ptr().as_ptr(),
					array_layout(&self.item_layout, self.capacity)
						.expect("array layout should be valid"),
					new_layout.size(),
				)
			};

			self.data = NonNull::new(new_data).unwrap_or_else(|| handle_alloc_error(new_layout));
		}
		self.capacity = new_capacity;
	}


	#[inline]
	pub fn len(&self) -> usize {
		self.len
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	#[inline]
	pub fn capacity(&self) -> usize {
		self.capacity
	}


	#[inline]
	pub unsafe fn get_ptr(&self) -> NonNull<u8> {
		self.data
	}

	#[inline]
	pub unsafe fn push_uninit(&mut self) -> usize {
		self.reserve_exact(1);
		let index = self.len;
		self.len += 1;
		index
	}

	#[inline]
	pub unsafe fn get_unchecked(&self, index: usize) -> *mut u8 {
		debug_assert!(index < self.len());
		self.get_ptr().as_ptr().add(index * self.item_layout.size())
	}

	#[inline]
	pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> *mut u8 {
		debug_assert!(index < self.len());
		self.get_ptr().as_ptr().add(index * self.item_layout.size())
	}

	pub unsafe fn push_element<T>(&mut self, element: T) {
		let index = self.push_uninit();
		let ptr = self.get_unchecked(index) as *mut T;
		ptr::write(ptr,element);
	}

	pub fn clear(&mut self) {
		let len = self.len;
		// We set len to 0 _before_ dropping elements for unwind safety. This ensures we don't
		// accidentally drop elements twice in the event of a drop impl panicking.
		self.len = 0;
		for i in 0..len {
			unsafe {
				// NOTE: this doesn't use self.get_unchecked(i) because the debug_assert on index
				// will panic here due to self.len being set to 0
				let ptr = self.get_ptr().as_ptr().add(i * self.item_layout.size());
				(self.drop)(ptr);
			}
		}
	}

	#[inline]
	pub unsafe fn swap_remove_and_forget_unchecked(&mut self, index: usize) -> *mut u8 {
		debug_assert!(index < self.len());
		let last = self.len - 1;
		let swap_scratch = self.swap_scratch.as_ptr();
		ptr::copy_nonoverlapping(
			self.get_unchecked(index),
			swap_scratch,
			self.item_layout.size(),
		);
		ptr::copy(
			self.get_unchecked(last),
			self.get_unchecked(index),
			self.item_layout.size(),
		);
		self.len -= 1;
		swap_scratch
	}

	#[inline]
	pub unsafe fn initialize_unchecked(&mut self, index: usize, value: *mut u8) {
		debug_assert!(index < self.len());
		let ptr = self.get_unchecked(index);
		ptr::copy_nonoverlapping(value, ptr, self.item_layout.size());
	}
}

impl Drop for BlobVec {
	fn drop(&mut self) {
		self.clear();
		let array_layout =
			array_layout(&self.item_layout, self.capacity).expect("array layout should be valid");
		if array_layout.size() > 0 {
			unsafe {
				std::alloc::dealloc(self.get_ptr().as_ptr(), array_layout);
				std::alloc::dealloc(self.swap_scratch.as_ptr(), self.item_layout);
			}
		}
	}
}

unsafe impl Send for BlobVec {}
unsafe impl Sync for BlobVec {}

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

#[derive(Debug, Clone)]
pub struct Column {
	pub data: BlobVec
}

impl Column {
	pub fn new<T: 'static>(capacity: usize) -> Self {
		let layout = Layout::new::<T>();
		let drop_fn = |ptr: *mut u8| unsafe {
			ptr::drop_in_place(ptr as *mut T);
		};
		Self {
			data: BlobVec::new(layout, drop_fn, capacity),
		}
	}

	pub fn data(&self) -> BlobVec {
		self.data.clone()
	}

	pub fn push<T: 'static>(&mut self, item: T) {
		assert_eq!(TypeId::of::<T>(), TypeId::of::<T>(), "Type mismatch");
		unsafe {
			let index = self.data.push_uninit();
			let ptr = self.data.get_unchecked(index);
			ptr::write(ptr as *mut T, item);
		}
	}

	pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
		assert_eq!(TypeId::of::<T>(), TypeId::of::<T>(), "Type mismatch");
		if index >= self.data.len() {
			return None;
		}
		unsafe {
			let ptr = self.data.get_unchecked(index);
			Some(&*(ptr as *const T))
		}
	}

	pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
		assert_eq!(TypeId::of::<T>(), TypeId::of::<T>(), "Type mismatch");

		if index >= self.data.len() {
			return None;
		}

		// Access the element at the given index
		unsafe {
			let ptr = self.data.get_unchecked(index);
			// Convert the pointer to a mutable reference and return it
			Some(&mut *(ptr as *mut T))
		}
	}

	pub fn remove<T: 'static>(&mut self, index: usize) -> Option<T> {
		assert_eq!(TypeId::of::<T>(), TypeId::of::<T>(), "Type mismatch");
		if index >= self.data.len() {
			return None;
		}
		unsafe {
			let ptr = self.data.swap_remove_and_forget_unchecked(index);
			Some(ptr::read(ptr as *const T))
		}
	}

	fn swap(&mut self, index1: usize, index2: usize) {
		assert!(index1 < self.data.len() && index2 < self.data.len(), "Index out of bounds");

		unsafe {
			let ptr1 = self.data.get_unchecked(index1);
			let ptr2 = self.data.get_unchecked(index2);

			let mut temp = MaybeUninit::<u8>::uninit();

			// Swap the elements at index1 and index2
			ptr::copy_nonoverlapping(ptr1, temp.as_mut_ptr(), self.data.item_layout.size());
			ptr::copy_nonoverlapping(ptr2, ptr1, self.data.item_layout.size());
			ptr::copy_nonoverlapping(temp.as_ptr(), ptr2, self.data.item_layout.size());
		}
	}
}
