use crate::Column;

#[derive(Debug, Clone)]
pub struct SparseSet {
    sparse: Vec<Option<Vec<Option<usize>>>>,
    dense: Column,
    page_size: usize,
}

impl SparseSet {
    pub fn new<T: 'static>(capacity: usize, page_size: usize) -> Self {
        Self {
            sparse: Vec::new(),
            dense: Column::new::<T>(capacity),
            page_size,
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
            // If there is already a mapping, overwrite the existing dense value instead of pushing.
            if let Some(sparse_index) = page_vec[index % self.page_size] {
                let _ = self.dense.set::<T>(sparse_index, value);
                return;
            }

            page_vec[index % self.page_size] = Some(self.dense.data.len());
        }

        self.dense.push(value);
    }

    pub fn remove<T: 'static>(&mut self, index: usize) -> Option<T> {
        if let Some(page_vec) = self
            .sparse
            .get(index / self.page_size)
            .and_then(|x| x.as_ref())
        {
            if let Some(sparse_index) = page_vec
                .get(index % self.page_size)
                .and_then(|x| x.as_ref())
            {
                let dense_index = *sparse_index;
                let last_index = self.dense.data.len() - 1;
                if dense_index != last_index {
                    self.dense.swap(dense_index, last_index);
                    if let Some(page_vec) = self
                        .sparse
                        .get_mut(last_index / self.page_size)
                        .and_then(|x| x.as_mut())
                    {
                        page_vec[last_index % self.page_size] = Some(dense_index);
                    }
                }
                if let Some(page_vec) = self
                    .sparse
                    .get_mut(index / self.page_size)
                    .and_then(|x| x.as_mut())
                {
                    page_vec[index % self.page_size] = None;
                }
                return self.dense.remove::<T>(last_index);
            }
        }
        None
    }

    /// Removes an element by external index without knowing its type. Returns true if something was removed.
    pub fn remove_any(&mut self, index: usize) -> bool {
        if let Some(page_vec) = self
            .sparse
            .get(index / self.page_size)
            .and_then(|x| x.as_ref())
        {
            if let Some(sparse_index) = page_vec
                .get(index % self.page_size)
                .and_then(|x| x.as_ref())
            {
                let dense_index = *sparse_index;
                let last_dense_index = self.dense.data.len() - 1;

                if dense_index != last_dense_index {
                    if let Some(ext_index) = self.find_external_index(last_dense_index) {
                        if let Some(page_vec) = self
                            .sparse
                            .get_mut(ext_index / self.page_size)
                            .and_then(|x| x.as_mut())
                        {
                            page_vec[ext_index % self.page_size] = Some(dense_index);
                        }
                    }
                    self.dense.swap(dense_index, last_dense_index);
                }

                if let Some(page_vec) = self
                    .sparse
                    .get_mut(index / self.page_size)
                    .and_then(|x| x.as_mut())
                {
                    page_vec[index % self.page_size] = None;
                }

                self.dense.remove_any(dense_index);
                return true;
            }
        }
        false
    }

    /// Finds the external index that maps to a given dense index.
    fn find_external_index(&self, dense_index: usize) -> Option<usize> {
        for (page_idx, page_opt) in self.sparse.iter().enumerate() {
            if let Some(page) = page_opt {
                for (offset, entry) in page.iter().enumerate() {
                    if let Some(idx) = entry {
                        if *idx == dense_index {
                            return Some(page_idx * self.page_size + offset);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
        if let Some(page_vec) = self
            .sparse
            .get(index / self.page_size)
            .and_then(|x| x.as_ref())
        {
            if let Some(sparse_index) = page_vec
                .get(index % self.page_size)
                .and_then(|x| x.as_ref())
            {
                self.dense.get::<T>(*sparse_index)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
        if let Some(page_vec) = self
            .sparse
            .get(index / self.page_size)
            .and_then(|x| x.as_ref())
        {
            if let Some(sparse_index) = page_vec
                .get(index % self.page_size)
                .and_then(|x| x.as_ref())
            {
                self.dense.get_mut::<T>(*sparse_index)
            } else {
                None
            }
        } else {
            None
        }
    }
}
