use crate::EntityId;
use comet_structs::{Column, ComponentSet};
use std::alloc::Layout;
use std::any::TypeId;
use std::collections::HashMap;

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
    entities: Vec<EntityId>,
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
            entities: Vec::new(),
            columns,
        }
    }

    pub fn set(&self) -> &ComponentSet {
        &self.set
    }

    pub fn types(&self) -> &[TypeId] {
        &self.types
    }

    pub fn column_index(&self, type_id: TypeId) -> Option<usize> {
        self.type_to_index.get(&type_id).copied()
    }

    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub fn columns_mut(&mut self) -> &mut [Column] {
        &mut self.columns
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn push_entity(&mut self, entity: EntityId) -> usize {
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
        component_index: &HashMap<TypeId, usize>,
    ) -> usize {
        if let Some(&id) = self.index.get(&set) {
            return id;
        }

        let mut types = set.to_vec();
        types.sort_by_key(|t| component_index.get(t).copied().unwrap_or(usize::MAX));

        let mut columns = Vec::with_capacity(types.len());
        for type_id in &types {
            let info = component_info.get(type_id).unwrap_or_else(|| {
                panic!(
                    "Component with TypeId {:?} not registered before archetype creation",
                    type_id
                )
            });
            columns.push(Column::new_raw(
                info.type_id,
                info.layout,
                info.drop_fn,
                0,
            ));
        }

        let archetype = Archetype::new(set.clone(), types, columns);
        let id = self.archetypes.len();
        self.archetypes.push(archetype);
        self.index.insert(set, id);
        id
    }
}
