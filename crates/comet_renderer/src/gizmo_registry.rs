use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use comet_ecs::{Component, Entity, Scene, Transform};
use comet_gizmos::{Gizmo, GizmoBuffer};

type DrawFn = Box<dyn Fn(Entity, &Scene, &mut GizmoBuffer) + Send + Sync>;

#[derive(Default)]
pub struct GizmoRegistry {
    enabled: HashMap<TypeId, (HashSet<Entity>, DrawFn)>,
}

impl GizmoRegistry {
    pub fn new() -> Self {
        Self { enabled: HashMap::new() }
    }

    pub fn show<C: Component + Gizmo + 'static>(&mut self, entity: Entity) {
        self.enabled
            .entry(C::type_id())
            .or_insert_with(|| {
                let draw: DrawFn = Box::new(|entity, scene, buffer| {
                    if let (Some(comp), Some(transform)) = (
                        scene.get_component::<C>(entity),
                        scene.get_component::<Transform>(entity),
                    ) {
                        comp.draw_gizmo(transform.position(), transform.rotation(), transform.scale(), buffer);
                    }
                });
                (HashSet::new(), draw)
            })
            .0
            .insert(entity);
    }

    pub fn hide<C: Component + Gizmo + 'static>(&mut self, entity: Entity) {
        if let Some((set, _)) = self.enabled.get_mut(&C::type_id()) {
            set.remove(&entity);
        }
    }

    #[allow(dead_code)]
    pub fn is_enabled<C: Component + Gizmo + 'static>(&self, entity: Entity) -> bool {
        self.enabled
            .get(&C::type_id())
            .map_or(false, |(set, _)| set.contains(&entity))
    }

    pub fn flush(&self, scene: &Scene, buffer: &mut GizmoBuffer) {
        for (entities, draw_fn) in self.enabled.values() {
            for &entity in entities {
                draw_fn(entity, scene, buffer);
            }
        }
    }
}
