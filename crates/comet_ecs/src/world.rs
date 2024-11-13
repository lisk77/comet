use std::any::TypeId;
use bit_set::BitSet;
use crate::{
	Entity,
	Component,
	Transform2D,
	Transform3D,
	ComponentStorage,
	SparseSet,
	IdQueue,
	Archetypes,
	ComponentSet
};
use comet_log::*;

pub struct World {
	dimension: String,
	id_queue: IdQueue,
	next_id: u32,
	entities: Vec<Option<Entity>>,
	components: ComponentStorage,
	archetypes: Archetypes
}

impl World {
	pub fn new(application: &str) -> Self {
		let mut component_storage = ComponentStorage::new();
		match application {
			"2D" => component_storage.register_component::<Transform2D>(0),
			"3D" => component_storage.register_component::<Transform3D>(0),
			_ => {}
		}

		Self {
			dimension: application.to_string(),
			id_queue: IdQueue::new(),
			next_id: 0,
			entities: Vec::new(),
			components: component_storage,
			archetypes: Archetypes::new()
		}
	}

	pub fn active_entities(&self) -> u32 {
		self.entities.len() as u32 - self.id_queue.size()
	}

	fn get_next_id(&mut self) {
		if self.id_queue.is_empty() {
			self.next_id = self.entities.len() as u32;
			return;
		}
		if self.next_id > self.id_queue.front().unwrap() || self.entities[self.next_id as usize] != None {
			self.next_id = self.id_queue.dequeue().unwrap();
		}
	}

	pub fn dimension(&self) -> &String {
		&self.dimension
	}

	pub fn id_queue(&self) -> &IdQueue {
		&self.id_queue
	}

	pub fn entities(&self) -> &Vec<Option<Entity>> {
		&self.entities
	}

	pub fn components(&self) -> &ComponentStorage {
		&self.components
	}

	pub fn components_mut(&mut self) -> &mut ComponentStorage {
		&mut self.components
	}

	pub fn new_entity(&mut self) -> u32 {
		let id = self.next_id;
		if (self.next_id as usize) >= self.entities.len() {
			self.entities.push(Some(Entity::new(self.next_id)));
			match self.dimension.as_str() {
				"2D" => self.add_component::<Transform2D>(self.next_id as usize, Transform2D::new()),
				"3D" => self.add_component::<Transform3D>(self.next_id as usize, Transform3D::new()),
				_ => {}
			}
			self.get_next_id();
			return id;
		}
		self.entities[self.next_id as usize] = Some(Entity::new(self.next_id));
		println!("{:?}", self.dimension);
		match self.dimension.as_str() {
			"2D" => self.add_component::<Transform2D>(self.next_id as usize, Transform2D::new()),
			"3D" => self.add_component::<Transform3D>(self.next_id as usize, Transform3D::new()),
			_ => {}
		}
		self.get_next_id();
		id
	}

	pub fn get_entity(&self, entity_id: usize) -> &Entity {
		assert_ne!(self.entities.get(entity_id), None, "There is no entity with this ID ({}) in the world!", entity_id);
		self.entities.get(entity_id).unwrap().as_ref().unwrap()
	}

	pub fn get_entity_mut(&mut self, entity_id: usize) -> &mut Entity {
		assert_ne!(self.entities.get(entity_id), None, "There is no entity with this ID ({}) in the world!", entity_id);
		self.entities.get_mut(entity_id).unwrap().as_mut().unwrap()
		//self.entities.get_mut(id).unwrap()
	}

	pub fn delete_entity(&mut self, entity_id: usize) {
		self.entities[entity_id] = None;
		//self.get_entity(id);
		for (key, value) in self.components.iter_mut() {
			value.remove::<u8>(entity_id);
		}
		self.id_queue.sorted_enqueue(entity_id as u32);
		self.get_next_id();
		self.remove_entity_from_archetype_subsets(entity_id as u32, self.get_component_set(entity_id));
		info!("Deleted entity! ID: {}", entity_id);
	}

	fn create_archetype(&mut self, components: ComponentSet) {
		self.archetypes.create_archetype(components);
	}

	fn remove_archetype(&mut self, components: ComponentSet) {
		self.archetypes.remove_archetype(&components);
	}

	fn remove_archetype_subsets(&mut self, components: ComponentSet) {
		let component_sets = self.archetypes.component_sets();
		let keys: Vec<ComponentSet> = component_sets.iter()
			.enumerate()
			.filter_map(|(i, &ref elem)| if elem.is_subset(&components) { Some(i) } else { None })
			.collect::<Vec<usize>>()
			.iter()
			.map(|index| component_sets[*index].clone())
			.collect::<Vec<ComponentSet>>();

		for key in keys {
			self.remove_archetype(key.clone());
		}
	}

	fn add_entity_to_archetype(&mut self, entity_id: u32, components: ComponentSet) {
		self.archetypes.add_entity_to_archetype(&components, entity_id);
	}

	fn remove_entity_from_archetype(&mut self, entity_id: u32, components: ComponentSet) {
		self.archetypes.remove_entity_from_archetype(&components, entity_id);
	}

	fn remove_entity_from_archetype_subsets(&mut self, entity_id: u32, components: ComponentSet) {
		let component_sets = self.archetypes.component_sets();
		let keys: Vec<ComponentSet> = component_sets.iter()
			.enumerate()
			.filter_map(|(i, &ref elem)| if elem.is_subset(&components) { Some(i) } else { None })
			.collect::<Vec<usize>>()
			.iter()
			.map(|index| component_sets[*index].clone())
			.collect::<Vec<ComponentSet>>();

		for key in keys {
			self.remove_entity_from_archetype(entity_id, key.clone());
			if self.archetypes.get_archetype(&key).unwrap().len() == 0 {
				self.archetypes.remove_archetype(&key);
			}
		}
	}

	fn get_component_set(&self, entity_id: usize) -> ComponentSet {
		let components = self.entities.get(entity_id).unwrap().as_ref().unwrap().get_components().iter().collect::<Vec<usize>>();
		let type_ids = components.iter().map(|index| self.components.keys[*index]).collect::<Vec<TypeId>>();
		ComponentSet::from_ids(type_ids)
	}

	pub fn register_component<T: Component + 'static>(&mut self) {
		self.components.register_component::<T>(self.entities.len());
		self.create_archetype(ComponentSet::from_ids(vec![T::type_id()]));
		info!("Registered component: {}", T::type_name());
	}

	pub fn deregister_component<T: Component + 'static>(&mut self) {
		self.components.deregister_component::<T>();
		info!("Deregistered component: {}", T::type_name());
	}

	pub fn add_component<T: Component + 'static>(&mut self, entity_id: usize, component: T) {
		assert_ne!(self.entities.get(entity_id), None, "There is no entity with this ID ({}) in the world!", entity_id);
		self.components.set_component(entity_id, component);
		let component_index = self.components.keys.iter_mut().position(|x| *x == T::type_id()).unwrap();

		self.get_entity_mut(entity_id).add_component(component_index);

		if !self.archetypes.contains_archetype(&self.get_component_set(entity_id)) {
			self.create_archetype(self.get_component_set(entity_id));
		}
		self.add_entity_to_archetype(entity_id as u32, ComponentSet::from_ids(vec![T::type_id()]));
		if self.get_component_set(entity_id) != ComponentSet::from_ids(vec![T::type_id()]) {
			self.add_entity_to_archetype(entity_id as u32, self.get_component_set(entity_id));
		}
		info!("Added component {} to entity {}", T::type_name(), entity_id);
	}

	pub fn remove_component<T: Component + 'static>(&mut self, entity_id: usize) {
		self.components.remove_component::<T>(entity_id);
		self.remove_entity_from_archetype_subsets(entity_id as u32, self.get_component_set(entity_id));
		info!("Removed component {} from entity {}", T::type_name(), entity_id);
	}

	pub fn get_component<T: Component + 'static>(&self, entity_id: usize) -> &T {
		assert_ne!(self.entities.get(entity_id), None, "There is no entity with this ID ({}) in the world!", entity_id);
		//assert_ne!(self.components.get_component::<T>(entity_id), None, "There is no component {} bound to the entity {} in the world!", T::type_name(), entity_id);
		self.components.get_component::<T>(entity_id).unwrap()
	}

	pub fn get_component_mut<T: Component + 'static>(&mut self, entity_id: usize) -> &mut T {
		assert_ne!(self.entities.get(entity_id), None, "There is no entity with this ID ({}) in the world!", entity_id);
		assert!(self.components.get_component::<T>(entity_id) != None, "There is no component {} bound to the entity {} in the world!", T::type_name(), entity_id);
		self.components.get_component_mut::<T>(entity_id).unwrap()
	}

	pub fn get_entities_with(&self, components: ComponentSet) -> Vec<u32> {
		assert!(self.archetypes.contains_archetype(&components), "The given components {:?} are not registered in the world!", components);
		//debug!(format!("Querying entities with components: {:?}", components));
		self.archetypes.get_archetype(&components).unwrap().clone()
	}
}
