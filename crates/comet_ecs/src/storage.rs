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

pub struct IterMut<'a, K, V> {
	keys_iter: std::slice::IterMut<'a, K>,
	values_iter: std::slice::IterMut<'a, V>,
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

#[derive(Debug, Clone)]
pub struct ComponentStorage {
	index_map: HashMap<TypeId, usize>,
	pub(crate) keys: Vec<TypeId>,
	components: Vec<SparseSet>
}

impl ComponentStorage {
	pub fn new() -> Self {
		Self {
			index_map: HashMap::new(),
			keys: Vec::new(),
			components: Vec::new()
		}
	}

	pub fn keys(&self) -> &Vec<TypeId> {
		&self.keys
	}

	pub fn contains_component(&self, type_id: &TypeId) -> bool {
		self.keys.contains(type_id)
	}

	pub fn get<T: Component + 'static>(&self) -> Option<&SparseSet> {
		self.components.get(*self.index_map.get(&T::type_id()).unwrap())
	}

	pub fn set<T: Component + 'static>(&mut self, sparse_set: SparseSet) {
		let _ = self.components.get_mut(*self.index_map.get(&T::type_id()).unwrap());
	}

	pub fn register_component<T: Component + 'static>(&mut self, capacity: usize) {
		//self.storage.insert(T::type_id(), SparseSet::new::<T>(capacity));
		assert!(!self.keys.contains(&T::type_id()), "This component ({}) is already registered!", T::type_name());
		self.keys.push(T::type_id());
		self.index_map.insert(T::type_id(), self.keys.len()-1);
		self.components.push(SparseSet::new::<T>(capacity));
	}

	pub fn get_component<T: Component + 'static>(&self, entity_id: usize) -> Option<&T> {
		//self.storage.get(&T::type_id()).unwrap().get::<T>(*entity.id() as usize)
		self.components.get(*self.index_map.get(&T::type_id()).unwrap()).unwrap().get::<T>(entity_id)
	}

	pub fn get_component_mut<T: Component + 'static>(&mut self, entity_id: usize) -> Option<&mut T> {
		self.components.get_mut(*self.index_map.get(&T::type_id()).unwrap()).unwrap().get_mut::<T>(entity_id)
	}

	pub fn set_component<T: Component + 'static>(&mut self, entity_id: usize, component: T) {
		let index = *self.index_map.get(&T::type_id()).unwrap();
		let sparse_set = self.components.get_mut(index).unwrap();

		// Check if a component already exists for this entity
		if let Some(existing_component) = sparse_set.get_mut::<T>(entity_id) {
			// Explicitly drop the existing component before overwriting it
			std::mem::drop(existing_component);
		}

		// Set the new component
		sparse_set.set(entity_id, component);
	}

	pub fn deregister_component<T: Component + 'static>(&mut self) {
		let type_id = T::type_id();
		if let Some(&index) = self.index_map.get(&type_id) {
			// Before removing the SparseSet, ensure all elements are properly dropped
			let sparse_set = self.components.get_mut(index).unwrap();
			for i in 0..sparse_set.sparse.len() {
				if sparse_set.sparse[i].is_some() {
					sparse_set.remove::<T>(i);
				}
			}

			self.components.remove(index);
			self.index_map.remove(&type_id);
			self.keys.retain(|&k| k != type_id);
		}
	}

	pub fn remove_component<T: Component + 'static>(&mut self, entity_id: usize) {
		if let Some(index) = self.index_map.get(&T::type_id()) {
			let sparse_set = self.components.get_mut(*index).unwrap();
			sparse_set.remove::<T>(entity_id);
		}
	}

	pub(crate) fn get_dense_list_as_vec<T: Component + Clone + 'static>(&self) -> Option<Vec<T>> {
		let mut resulting_vec: Vec<T> = Vec::new();

		if let Some(sparse_set) = self.components.get(*self.index_map.get(&T::type_id()).unwrap()) {
			for i in 0..sparse_set.dense.data.len() {
				let item: T = sparse_set.dense.get::<T>(i)?.clone();
				resulting_vec.push(item);
			}
			Some(resulting_vec)
		} else {
			None
		}
	}

	pub fn iter_mut(&mut self) -> IterMut<'_, TypeId, SparseSet> {
		IterMut {
			keys_iter: self.keys.iter_mut(),
			values_iter: self.components.iter_mut(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSet {
	set: HashSet<TypeId>
}

impl ComponentSet {
	pub fn new() -> Self {
		Self {
			set: HashSet::new()
		}
	}

	pub fn from_ids(ids: Vec<TypeId>) -> Self {
		Self {
			set: ids.into_iter().collect()
		}
	}

	pub fn is_subset(&self, other: &ComponentSet) -> bool {
		self.set.is_subset(&other.set)
	}
}

impl Hash for ComponentSet {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let mut types: Vec<TypeId> = self.set.iter().cloned().collect();
		types.sort();
		types.hash(state);
	}
}

#[derive(Debug)]
pub struct Archetypes {
	archetypes: HashMap<ComponentSet, Vec<u32>>
}

impl Archetypes {
	pub fn new() -> Self {
		Self {
			archetypes: HashMap::new()
		}
	}

	pub fn component_sets(&self) -> Vec<ComponentSet> {
		self.archetypes.keys().cloned().collect()
	}

	pub fn create_archetype(&mut self, components: ComponentSet) {
		self.archetypes.insert(components, Vec::new());
	}

	pub fn get_archetype(&self, components: &ComponentSet) -> Option<&Vec<u32>> {
		self.archetypes.get(components)
	}

	pub fn get_archetype_mut(&mut self, components: &ComponentSet) -> Option<&mut Vec<u32>> {
		self.archetypes.get_mut(components)
	}

	pub fn add_entity_to_archetype(&mut self, components: &ComponentSet, entity: u32) {
		if let Some(archetype) = self.archetypes.get_mut(components) {
			archetype.push(entity);
		}
	}

	pub fn remove_entity_from_archetype(&mut self, components: &ComponentSet, entity: u32) {
		if let Some(archetype) = self.archetypes.get_mut(components) {
			archetype.retain(|&id| id != entity);
		}
	}

	pub fn remove_archetype(&mut self, components: &ComponentSet) {
		self.archetypes.remove(components);
	}

	pub fn contains_archetype(&self, components: &ComponentSet) -> bool {
		self.archetypes.contains_key(components)
	}
}
