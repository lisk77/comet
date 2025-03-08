use crate::Column;
use std::hash::{
	Hash,
};

#[derive(Debug, Clone)]
pub struct SparseSet {
	sparse: Vec<Option<Vec<Option<usize>>>>,
	dense: Column,
	page_size: usize
}

impl SparseSet {
	pub fn new<T: 'static>(capacity: usize, page_size: usize) -> Self {
		Self {
			sparse: Vec::new(),
			dense: Column::new::<T>(capacity),
			page_size
		}
	}

	pub fn insert<T: 'static>(&mut self, index: usize, value: T) {
		let page = index / self.page_size;

		if page >= self.sparse.len() {
			self.sparse.resize(page + 1, None);
		}

		if self.sparse[page].is_none() {
			self.sparse[page] = Some(vec![None; self.page_size]);
		}

		if let Some(page_vec) = &mut self.sparse[page] {
			page_vec[index % self.page_size] = Some(self.dense.data.len());
		}

		self.dense.push(value);
	}

	pub fn remove<T: 'static>(&mut self, index: usize) -> Option<T> {
		if let Some(page_vec) = self.sparse.get(index / self.page_size).and_then(|x| x.as_ref()) {
			if let Some(sparse_index) = page_vec.get(index % self.page_size).and_then(|x| x.as_ref()) {
				let dense_index = *sparse_index;
				let last_index = self.dense.data.len() - 1;
				if dense_index != last_index {
					self.dense.swap(dense_index, last_index);
					if let Some(page_vec) = self.sparse.get_mut(last_index / self.page_size).and_then(|x| x.as_mut()) {
						page_vec[last_index % self.page_size] = Some(dense_index);
					}
				}
				if let Some(page_vec) = self.sparse.get_mut(index / self.page_size).and_then(|x| x.as_mut()) {
					page_vec[index % self.page_size] = None;
				}
				return self.dense.remove::<T>(last_index);
			}
		}
		None
	}

	pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
		if let Some(page_vec) = self.sparse.get(index / self.page_size).and_then(|x| x.as_ref()) {
			if let Some(sparse_index) = page_vec.get(index % self.page_size).and_then(|x| x.as_ref()) {
				self.dense.get::<T>(*sparse_index)
			}
			else {
				None
			}
		}
		else {
			None
		}
	}

	pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
		if let Some(page_vec) = self.sparse.get(index / self.page_size).and_then(|x| x.as_ref()) {
			if let Some(sparse_index) = page_vec.get(index % self.page_size).and_then(|x| x.as_ref()) {
				self.dense.get_mut::<T>(*sparse_index)
			}
			else {
				None
			}
		}
		else {
			None
		}
	}
}