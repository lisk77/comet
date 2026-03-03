#[derive(Debug)]
pub struct SparseSet<T> {
    sparse: Vec<Option<Vec<Option<usize>>>>,
    dense: Vec<T>,
    dense_to_external: Vec<usize>,
    page_size: usize,
}

impl<T> SparseSet<T> {
    pub fn new(capacity: usize, page_size: usize) -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::with_capacity(capacity),
            dense_to_external: Vec::with_capacity(capacity),
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
        self.dense_to_external.push(index);
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
            self.dense_to_external.swap(dense_index, last_index);

            let swapped_external = self.dense_to_external[dense_index];
            if let Some(page_vec) = self
                .sparse
                .get_mut(swapped_external / self.page_size)
                .and_then(|x| x.as_mut())
            {
                page_vec[swapped_external % self.page_size] = Some(dense_index);
            }
        }

        if let Some(page_vec) = self
            .sparse
            .get_mut(index / self.page_size)
            .and_then(|x| x.as_mut())
        {
            page_vec[index % self.page_size] = None;
        }

        let _ = self.dense_to_external.pop();
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
}

#[cfg(test)]
mod tests {
    use super::SparseSet;

    #[test]
    fn remove_updates_swapped_back_reference() {
        let mut set = SparseSet::new(4, 16);
        set.insert(10, "a");
        set.insert(42, "b");
        set.insert(77, "c");

        let removed = set.remove(42);
        assert_eq!(removed, Some("b"));
        assert_eq!(set.get(42), None);
        assert_eq!(set.get(10), Some(&"a"));
        assert_eq!(set.get(77), Some(&"c"));
        assert_eq!(set.dense_len(), 2);
    }

    #[test]
    fn insert_existing_keeps_dense_index_stable() {
        let mut set = SparseSet::new(2, 8);
        set.insert(3, 10);
        let idx_before = set.dense_index(3);
        set.insert(3, 20);

        assert_eq!(set.get(3), Some(&20));
        assert_eq!(set.dense_index(3), idx_before);
        assert_eq!(set.dense_len(), 1);
    }
}
