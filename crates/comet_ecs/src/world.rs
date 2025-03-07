use std::any::TypeId;
use bit_set::BitSet;
use crate::{
	Entity,
	Component,
	Transform2D,
	Transform3D,
	IdQueue,
};
use comet_log::*;
use comet_structs::*;
use crate::archetypes::Archetypes;

#[derive(Clone)]
pub struct World {
	id_queue: IdQueue,
	next_id: u32,
	entities: Vec<Option<Entity>>,
	components: ComponentStorage,
	archetypes: Archetypes
}

impl World {
	pub fn new() -> Self {
		Self {
			id_queue: IdQueue::new(),
			next_id: 0,
			entities: Vec::new(),
			components: ComponentStorage::new(),
			archetypes: Archetypes::new()
		}
	}

	/// Returns the number of how many entities exist in the current World.
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

	/// Retuns the `Vec` of `Option<Entity>` which contains all the entities in the current World.
	pub fn entities(&self) -> &Vec<Option<Entity>> {
		&self.entities
	}

	/// Creates a new entity and returns its ID.
	pub fn new_entity(&mut self) -> u32 {
		let id = self.next_id;
		if (self.next_id as usize) >= self.entities.len() {
			self.entities.push(Some(Entity::new(self.next_id)));
			self.get_next_id();
			info!("Created entity! ID: {}", id);
			return id;
		}
		self.entities[self.next_id as usize] = Some(Entity::new(self.next_id));
		self.get_next_id();
		info!("Created entity! ID: {}", id);
		id
	}

	/// Gets an immutable reference to an entity by its ID.
	pub fn get_entity(&self, entity_id: usize) -> Option<&Entity> {
		self.entities.get(entity_id).unwrap().as_ref()
	}

	/// Gets a mutable reference to an entity by its ID.
	pub fn get_entity_mut(&mut self, entity_id: usize) -> Option<&mut Entity> {
		self.entities.get_mut(entity_id).unwrap().as_mut()
	}

	/// Deletes an entity by its ID.
	pub fn delete_entity(&mut self, entity_id: usize) {
		self.remove_entity_from_archetype_subsets(entity_id as u32, self.get_component_set(entity_id));
		self.entities[entity_id] = None;
		info!("Deleted entity! ID: {}", entity_id);
		for (_, value) in self.components.iter_mut() {
			value.remove::<u8>(entity_id);

		}
		self.id_queue.sorted_enqueue(entity_id as u32);
		self.get_next_id();


	}

	fn create_archetype(&mut self, components: ComponentSet) {
		self.archetypes.create_archetype(components);
	}

	fn remove_archetype(&mut self, components: ComponentSet) {
		self.archetypes.remove_archetype(&components);
	}

	fn get_keys(&self, components: ComponentSet) -> Vec<ComponentSet> {
		let component_sets = self.archetypes.component_sets();
		component_sets.iter()
			.enumerate()
			.filter_map(|(i, &ref elem)| if elem.is_subset(&components) { Some(i) } else { None })
			.collect::<Vec<usize>>()
			.iter()
			.map(|index| component_sets[*index].clone())
			.collect::<Vec<ComponentSet>>()
	}

	fn remove_archetype_subsets(&mut self, components: ComponentSet) {
		let keys = self.get_keys(components);

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
		let keys = self.get_keys(components);

		for key in keys {
			self.remove_entity_from_archetype(entity_id, key.clone());
			if self.archetypes.get_archetype(&key).unwrap().len() == 0 {
				self.archetypes.remove_archetype(&key);
			}
		}
		info!("Removed entity {} from all archetypes!", entity_id);
	}

	fn get_component_set(&self, entity_id: usize) -> ComponentSet {
		let components = match self.entities.get(entity_id) {
			Some(cmp) => match cmp.as_ref() {
				Some(e) => e.get_components().iter().collect::<Vec<usize>>(),
				None => {
					error!("This entity ({}) does not have any components!", entity_id);
					Vec::new()
				}
			},
			_ => {
				error!("This entity ({}) does not exist!", entity_id);
				Vec::new()
			}
		};

		let type_ids = components.iter().map(|index| self.components.keys()[*index]).collect::<Vec<TypeId>>();
		ComponentSet::from_ids(type_ids)
	}

	/// Registers a new component in the world.
	pub fn register_component<C: Component + 'static>(&mut self) {
		self.components.register_component::<C>(self.entities.len());
		self.create_archetype(ComponentSet::from_ids(vec![C::type_id()]));
		info!("Registered component: {}", C::type_name());
	}

	/// Deregisters a component from the world.
	pub fn deregister_component<C: Component + 'static>(&mut self) {
		self.components.deregister_component::<C>();
		info!("Deregistered component: {}", C::type_name());
	}

	/// Adds a component to an entity by its ID and an instance of the component.
	pub fn add_component<C: Component + 'static>(&mut self, entity_id: usize, component: C) {
		self.components.set_component(entity_id, component);
		let component_index = self.components.keys().iter_mut().position(|x| *x == C::type_id()).unwrap();

		self.get_entity_mut(entity_id).unwrap().add_component(component_index);

		if !self.archetypes.contains_archetype(&self.get_component_set(entity_id)) {
			self.create_archetype(self.get_component_set(entity_id));
		}
		self.add_entity_to_archetype(entity_id as u32, ComponentSet::from_ids(vec![C::type_id()]));
		if self.get_component_set(entity_id) != ComponentSet::from_ids(vec![C::type_id()]) {
			self.add_entity_to_archetype(entity_id as u32, self.get_component_set(entity_id));
		}
		info!("Added component {} to entity {}!", C::type_name(), entity_id);
	}

	/// Removes a component from an entity by its ID.
	pub fn remove_component<C: Component + 'static>(&mut self, entity_id: usize) {
		self.components.remove_component::<C>(entity_id);
		self.remove_entity_from_archetype_subsets(entity_id as u32, self.get_component_set(entity_id));
		info!("Removed component {} from entity {}!", C::type_name(), entity_id);
	}

	/// Returns a reference to a component of an entity by its ID.
	pub fn get_component<C: Component + 'static>(&self, entity_id: usize) -> Option<&C> {
		self.components.get_component::<C>(entity_id)
	}

	pub fn get_component_mut<C: Component + 'static>(&mut self, entity_id: usize) -> Option<&mut C> {
		self.components.get_component_mut::<C>(entity_id)
	}

	/// Returns a list of entities that have the given components.
	pub fn get_entities_with(&self, components: ComponentSet) -> Vec<usize> {
		if self.archetypes.contains_archetype(&components) {
			return self.archetypes.get_archetype(&components).unwrap().clone().iter().map(|x| *x as usize).collect();
		}
		error!("The given components {:?} are not registered in the world!", components);
		Vec::new()
	}
}
