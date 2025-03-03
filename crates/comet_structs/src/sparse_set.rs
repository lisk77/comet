use crate::Column;
use std::hash::{
	Hash,
	Hasher
};
use std::ptr;

#[derive(Debug, Clone)]
pub struct SparseSet {
	sparse: Vec<Option<usize>>,
	dense: Column,
	len: usize
}

impl SparseSet {
	pub fn new<T: 'static>(capacity: usize) -> Self {
		Self {
			sparse: Vec::with_capacity(capacity),
			dense: Column::new::<T>(capacity),
			len: 0
		}
	}

	pub fn set<T: 'static>(&mut self, index: usize, element: T) {
		if index >= self.sparse.len() {
			self.sparse.resize_with(index + 1, || None);
		}

		if let Some(column_index) = self.sparse[index] {
			// Explicitly drop the existing component before replacing it
			unsafe {
				let existing_ptr = self.dense.data.get_unchecked_mut(column_index) as *mut T;
				ptr::drop_in_place(existing_ptr);
				ptr::write(existing_ptr, element);
			}
		} else {
			let column_index = unsafe { self.dense.data.push_uninit() };
			unsafe {
				self.dense.data.initialize_unchecked(column_index, &element as *const T as *mut u8);
			}
			self.sparse[index] = Some(column_index);
			self.len += 1;
		}
	}

	pub fn remove<T: 'static>(&mut self, index: usize) -> Option<T> {
		if index >= self.sparse.len() || self.sparse[index] == None {
			return None;
		}

		let column_index = self.sparse[index];
		let element = unsafe {
			self.dense.data.swap_remove_and_forget_unchecked(column_index.unwrap())
		};

		self.sparse[index] = None;
		self.len -= 1;

		Some(unsafe { ptr::read(element as *const T) })
	}

	pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
		if index >= self.sparse.len() || self.sparse[index] == None {
			return None;
		}

		self.dense.get::<T>(index)
	}

	pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
		if index >= self.sparse.len() || self.sparse[index] == None {
			return None;
		}

		self.dense.get_mut::<T>(index)
	}
}
