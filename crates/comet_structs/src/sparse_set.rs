use crate::Column;
use std::hash::{
	Hash,
};

#[derive(Debug, Clone)]
pub struct SparseSet {
	pub sparse: Vec<Option<usize>>,
	pub dense: Column,
}

impl SparseSet {
	pub fn new<T: 'static>(capacity: usize) -> Self {
		Self {
			sparse: Vec::new(),
			dense: Column::new::<T>(capacity),
		}
	}

	pub fn insert<T: 'static>(&mut self, index: usize, value: T) {
		if index >= self.sparse.len() {
			self.sparse.resize(index + 1, None);
		}
		self.sparse[index] = Some(self.dense.data.len());
		self.dense.push(value);
	}

	pub fn remove<T: 'static>(&mut self, index: usize) -> Option<T>{
		if let Some(sparse_index) = self.sparse.get(index).and_then(|x| x.as_ref()) {
			let dense_index = *sparse_index;
			let last_index = self.dense.data.len() - 1;
			if dense_index != last_index {
				self.dense.swap(dense_index, last_index);
				if let Some(sparse) = self.sparse.get_mut(last_index) {
					*sparse = Some(dense_index);
				}
			}
			self.sparse[index] = None;
			self.dense.remove::<T>(last_index)
		}
		else {
			None
		}
	}

	pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
		match self.sparse.get(index).and_then(|x| x.as_ref()) {
			Some(sparse_index) => self.dense.get::<T>(*sparse_index),
			None => None,
		}
	}

	pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
		match self.sparse.get(index).and_then(|x| x.as_ref()) {
			Some(sparse_index) => self.dense.get_mut::<T>(*sparse_index),
			None => None,
		}
	}
}