pub struct IterMut<'a, K, V> {
	pub(crate) keys_iter: std::slice::IterMut<'a, K>,
	pub(crate) values_iter: std::slice::IterMut<'a, V>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
	type Item = (&'a mut K, &'a mut V);

	fn next(&mut self) -> Option<Self::Item> {
		match (self.keys_iter.next(), self.values_iter.next()) {
			(Some(key), Some(value)) => Some((key, value)),
			_ => None,
		}
	}
}