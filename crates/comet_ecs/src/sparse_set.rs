#[derive(Debug)]
pub struct SparseSet<T> {
    sparse: Vec<Option<Vec<Option<usize>>>>,
    dense: Vec<T>,
    page_size: usize,
}

impl<T> SparseSet<T> {
    pub fn new(capacity: usize, page_size: usize) -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::with_capacity(capacity),
            page_size,
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        let page = index / self.page_size;

        if page >= self.sparse.len() {
            self.sparse.resize(page + 1, None);
        }

        if self.sparse[page].is_none() {
            self.sparse[page] = Some(vec![None; self.page_size]);
        }

        if let Some(page_vec) = &mut self.sparse[page] {
            if let Some(sparse_index) = page_vec[index % self.page_size] {
                self.dense[sparse_index] = value;
                return;
            }

            page_vec[index % self.page_size] = Some(self.dense.len());
        }

        self.dense.push(value);
    }

    pub fn dense_index(&self, index: usize) -> Option<usize> {
        self.sparse_index(index)
    }

    pub fn dense_len(&self) -> usize {
        self.dense.len()
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        let dense_index = self.sparse_index(index)?;
        let last_index = self.dense.len() - 1;

        if dense_index != last_index {
            self.dense.swap(dense_index, last_index);
            if let Some(ext_index) = self.find_external_index(last_index) {
                if let Some(page_vec) = self
                    .sparse
                    .get_mut(ext_index / self.page_size)
                    .and_then(|x| x.as_mut())
                {
                    page_vec[ext_index % self.page_size] = Some(dense_index);
                }
            }
        }

        if let Some(page_vec) = self
            .sparse
            .get_mut(index / self.page_size)
            .and_then(|x| x.as_mut())
        {
            page_vec[index % self.page_size] = None;
        }

        self.dense.pop()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let dense_index = self.sparse_index(index)?;
        self.dense.get(dense_index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let dense_index = self.sparse_index(index)?;
        self.dense.get_mut(dense_index)
    }

    pub fn dense(&self) -> &[T] {
        &self.dense
    }

    fn sparse_index(&self, index: usize) -> Option<usize> {
        self.sparse
            .get(index / self.page_size)
            .and_then(|x| x.as_ref())
            .and_then(|page_vec| page_vec.get(index % self.page_size))
            .and_then(|x| *x)
    }

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
}
