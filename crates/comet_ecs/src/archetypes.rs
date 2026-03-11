use crate::Entity;
use comet_structs::{Column, ComponentSet};
use std::alloc::Layout;
use std::any::TypeId;
use std::collections::HashMap;

const DEFAULT_ARCHETYPE_CAPACITY: usize = 64;

#[derive(Clone)]
pub struct ComponentInfo {
    pub type_id: TypeId,
    pub layout: Layout,
    pub drop_fn: unsafe fn(*mut u8),
}

pub struct Archetype {
    set: ComponentSet,
    types: Vec<TypeId>,
    type_to_index: HashMap<TypeId, usize>,
    add_edges: HashMap<TypeId, usize>,
    remove_edges: HashMap<TypeId, usize>,
    entities: Vec<Entity>,
    columns: Vec<Column>,
}

impl Archetype {
    pub fn new(set: ComponentSet, types: Vec<TypeId>, columns: Vec<Column>) -> Self {
        let type_to_index = types
            .iter()
            .enumerate()
            .map(|(i, t)| (*t, i))
            .collect::<HashMap<_, _>>();
        Self {
            set,
            types,
            type_to_index,
            add_edges: HashMap::new(),
            remove_edges: HashMap::new(),
            entities: Vec::with_capacity(DEFAULT_ARCHETYPE_CAPACITY),
            columns,
        }
    }

    pub fn set(&self) -> &ComponentSet {
        &self.set
    }

    pub fn types(&self) -> &[TypeId] {
        &self.types
    }

    #[inline(always)]
    pub fn column_index(&self, type_id: TypeId) -> Option<usize> {
        self.type_to_index.get(&type_id).copied()
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn columns_mut(&mut self) -> &mut [Column] {
        &mut self.columns
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    #[inline(always)]
    pub fn push_entity(&mut self, entity: Entity) -> usize {
        let row = self.entities.len();
        self.entities.push(entity);
        row
    }

    pub fn swap_rows(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        self.entities.swap(a, b);
        for col in &mut self.columns {
            col.swap(a, b);
        }
    }

    pub fn pop_entity(&mut self) {
        let _ = self.entities.pop();
    }

    pub fn add_edge(&self, type_id: TypeId) -> Option<usize> {
        self.add_edges.get(&type_id).copied()
    }

    pub fn remove_edge(&self, type_id: TypeId) -> Option<usize> {
        self.remove_edges.get(&type_id).copied()
    }

    pub fn set_add_edge(&mut self, type_id: TypeId, target: usize) {
        self.add_edges.insert(type_id, target);
    }

    pub fn set_remove_edge(&mut self, type_id: TypeId, target: usize) {
        self.remove_edges.insert(type_id, target);
    }
}

pub struct Archetypes {
    archetypes: Vec<Archetype>,
    index: HashMap<ComponentSet, usize>,
}

impl Archetypes {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn get(&self, id: usize) -> &Archetype {
        &self.archetypes[id]
    }

    pub fn get_mut(&mut self, id: usize) -> &mut Archetype {
        &mut self.archetypes[id]
    }

    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        self.archetypes.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.archetypes.len()
    }

    pub fn get_two_mut(&mut self, a: usize, b: usize) -> (&mut Archetype, &mut Archetype) {
        assert!(a != b, "cannot mutably borrow the same archetype twice");
        if a < b {
            let (left, right) = self.archetypes.split_at_mut(b);
            (&mut left[a], &mut right[0])
        } else {
            let (left, right) = self.archetypes.split_at_mut(a);
            (&mut right[0], &mut left[b])
        }
    }

    pub fn get_or_create(
        &mut self,
        set: ComponentSet,
        component_info: &HashMap<TypeId, ComponentInfo>,
        component_registry: &[Option<TypeId>],
    ) -> usize {
        if let Some(&id) = self.index.get(&set) {
            return id;
        }

        let mut indices = set.to_vec();
        indices.sort_unstable();
        let mut types = Vec::with_capacity(indices.len());
        let mut columns = Vec::with_capacity(indices.len());
        for index in indices {
            let type_id = component_registry
                .get(index)
                .and_then(|entry| *entry)
                .unwrap_or_else(|| panic!("Component index {} not registered", index));
            let info = component_info.get(&type_id).unwrap_or_else(|| {
                panic!(
                    "Component with TypeId {:?} not registered before archetype creation",
                    type_id
                )
            });
            types.push(type_id);
            columns.push(Column::new_raw(
                info.type_id,
                info.layout,
                info.drop_fn,
                DEFAULT_ARCHETYPE_CAPACITY,
            ));
        }

        let archetype = Archetype::new(set.clone(), types, columns);
        let id = self.archetypes.len();
        self.archetypes.push(archetype);
        self.index.insert(set, id);
        id
    }

    pub fn get_or_create_add_edge(
        &mut self,
        from: usize,
        type_id: TypeId,
        component_info: &HashMap<TypeId, ComponentInfo>,
        component_index: &HashMap<TypeId, usize>,
        component_registry: &[Option<TypeId>],
    ) -> usize {
        if let Some(next) = self.archetypes[from].add_edge(type_id) {
            return next;
        }

        let index = component_index
            .get(&type_id)
            .copied()
            .unwrap_or_else(|| panic!("Component {:?} missing index", type_id));
        let mut next_set = self.archetypes[from].set().clone();
        next_set.insert(index);
        let to = self.get_or_create(next_set, component_info, component_registry);

        self.archetypes[from].set_add_edge(type_id, to);
        if from != to {
            self.archetypes[to].set_remove_edge(type_id, from);
        }
        to
    }

    pub fn get_or_create_remove_edge(
        &mut self,
        from: usize,
        type_id: TypeId,
        component_info: &HashMap<TypeId, ComponentInfo>,
        component_index: &HashMap<TypeId, usize>,
        component_registry: &[Option<TypeId>],
    ) -> usize {
        if let Some(next) = self.archetypes[from].remove_edge(type_id) {
            return next;
        }

        let index = component_index
            .get(&type_id)
            .copied()
            .unwrap_or_else(|| panic!("Component {:?} missing index", type_id));
        let mut next_set = self.archetypes[from].set().clone();
        next_set.remove(index);
        let to = self.get_or_create(next_set, component_info, component_registry);

        self.archetypes[from].set_remove_edge(type_id, to);
        if from != to {
            self.archetypes[to].set_add_edge(type_id, from);
        }
        to
    }
}
