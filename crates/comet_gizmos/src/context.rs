use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use comet_ecs::{Component, Entity};
use crate::Gizmo;

#[derive(Default)]
pub struct GizmoContext {
    enabled: HashMap<TypeId, HashSet<Entity>>,
}

impl GizmoContext {
    pub fn new() -> Self {
        Self { enabled: HashMap::new() }
    }

    pub fn show<C: Component + Gizmo + 'static>(&mut self, entity: Entity) {
        self.enabled.entry(C::type_id()).or_default().insert(entity);
    }

    pub fn hide<C: Component + Gizmo + 'static>(&mut self, entity: Entity) {
        if let Some(set) = self.enabled.get_mut(&C::type_id()) {
            set.remove(&entity);
        }
    }

    pub fn is_enabled<C: Component + Gizmo + 'static>(&self, entity: Entity) -> bool {
        self.enabled.get(&C::type_id())
            .map_or(false, |set| set.contains(&entity))
    }

    pub fn enabled_for<C: Component + Gizmo + 'static>(&self) -> Option<&HashSet<Entity>> {
        self.enabled.get(&C::type_id())
    }

    pub fn all_enabled(&self) -> &HashMap<TypeId, HashSet<Entity>> {
        &self.enabled
    }
}
