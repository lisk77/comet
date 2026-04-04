use super::*;
use crate::{ComponentTuple, Tick};
use std::any::TypeId;

macro_rules! impl_query_state_methods_scene_ref {
    () => {
        pub fn with<Co: Component>(mut self) -> Self {
            self.state.with_components.push(Co::type_id());
            self
        }

        pub fn without<Co: Component>(mut self) -> Self {
            self.state.without_components.push(Co::type_id());
            self
        }

        pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
            self.state.with_any_components.extend(Cs::type_ids());
            self
        }

        pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
            self.state.without_any_components.extend(Cs::type_ids());
            self
        }

        pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
            self.state.with_components.extend(Cs::type_ids());
            self
        }

        pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
            self.state.without_components.extend(Cs::type_ids());
            self
        }

        pub fn added<Co: Component>(mut self) -> Self {
            self.state
                .set_added_since_filter(Co::type_id(), self.scene.default_query_since_tick());
            self
        }

        pub fn changed<Co: Component>(mut self) -> Self {
            self.state
                .set_changed_since_filter(Co::type_id(), self.scene.default_query_since_tick());
            self
        }

        pub fn added_since<Co: Component>(mut self, tick: Tick) -> Self {
            self.state.set_added_since_filter(Co::type_id(), tick);
            self
        }

        pub fn changed_since<Co: Component>(mut self, tick: Tick) -> Self {
            self.state.set_changed_since_filter(Co::type_id(), tick);
            self
        }
    };
}

macro_rules! impl_query_state_methods_write_ptr {
    () => {
        pub fn with<Co: Component>(mut self) -> Self {
            self.state.with_components.push(Co::type_id());
            self
        }

        pub fn without<Co: Component>(mut self) -> Self {
            self.state.without_components.push(Co::type_id());
            self
        }

        pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
            self.state.with_any_components.extend(Cs::type_ids());
            self
        }

        pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
            self.state.without_any_components.extend(Cs::type_ids());
            self
        }

        pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
            self.state.with_components.extend(Cs::type_ids());
            self
        }

        pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
            self.state.without_components.extend(Cs::type_ids());
            self
        }

        pub fn added<Co: Component>(mut self) -> Self {
            self.state.set_added_since_filter(Co::type_id(), unsafe {
                (&*self.scene).default_query_since_tick()
            });
            self
        }

        pub fn changed<Co: Component>(mut self) -> Self {
            self.state.set_changed_since_filter(Co::type_id(), unsafe {
                (&*self.scene).default_query_since_tick()
            });
            self
        }

        pub fn added_since<Co: Component>(mut self, tick: Tick) -> Self {
            self.state.set_added_since_filter(Co::type_id(), tick);
            self
        }

        pub fn changed_since<Co: Component>(mut self, tick: Tick) -> Self {
            self.state.set_changed_since_filter(Co::type_id(), tick);
            self
        }
    };
}

macro_rules! impl_query_state_methods_scene_mut {
    () => {
        pub fn with<Co: Component>(mut self) -> Self {
            self.state.with_components.push(Co::type_id());
            self
        }

        pub fn without<Co: Component>(mut self) -> Self {
            self.state.without_components.push(Co::type_id());
            self
        }

        pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
            self.state.with_any_components.extend(Cs::type_ids());
            self
        }

        pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
            self.state.without_any_components.extend(Cs::type_ids());
            self
        }

        pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
            self.state.with_components.extend(Cs::type_ids());
            self
        }

        pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
            self.state.without_components.extend(Cs::type_ids());
            self
        }

        pub fn added<Co: Component>(mut self) -> Self {
            self.state
                .set_added_since_filter(Co::type_id(), self.scene.default_query_since_tick());
            self
        }

        pub fn changed<Co: Component>(mut self) -> Self {
            self.state
                .set_changed_since_filter(Co::type_id(), self.scene.default_query_since_tick());
            self
        }

        pub fn added_since<Co: Component>(mut self, tick: Tick) -> Self {
            self.state.set_added_since_filter(Co::type_id(), tick);
            self
        }

        pub fn changed_since<Co: Component>(mut self, tick: Tick) -> Self {
            self.state.set_changed_since_filter(Co::type_id(), tick);
            self
        }
    };
}

fn cached_single_plan_for(
    scene: &Scene,
    state: &QueryFilterState,
    type_id: TypeId,
) -> Vec<(usize, usize)> {
    scene.cached_single_plan(
        type_id,
        &state.with_components,
        &state.without_components,
        &state.with_any_components,
        &state.without_any_components,
    )
}

fn assert_unique_query_types(required: &[TypeId]) {
    assert!(
        !has_duplicate_type_ids(required),
        "query called with duplicate component types"
    );
}

fn for_each_matching_archetype(
    scene: &Scene,
    state: &QueryFilterState,
    primary_type: TypeId,
    required_types: &[TypeId],
    mut f: impl FnMut(&Scene, usize, usize),
) {
    assert_unique_query_types(required_types);
    for (arch_id, first_idx) in cached_single_plan_for(scene, state, primary_type) {
        f(scene, arch_id, first_idx);
    }
}

fn for_each_matching_archetype_mut(
    scene: &Scene,
    state: &QueryFilterState,
    primary_type: TypeId,
    required_types: &[TypeId],
    mut f: impl FnMut(&Scene, usize, usize),
) {
    assert_unique_query_types(required_types);
    for (arch_id, first_idx) in cached_single_plan_for(scene, state, primary_type) {
        f(scene, arch_id, first_idx);
    }
}

fn build_single_read_accesses<'a, P: ReadFetch<'a>>(
    scene: &'a Scene,
    state: &QueryFilterState,
) -> Vec<QueryAccess> {
    let mut accesses = Vec::new();
    for_each_matching_archetype(
        scene,
        state,
        P::type_id(),
        &[P::type_id()],
        |scene, arch_id, col_idx| {
            let arch = scene.archetypes().get(arch_id);
            let col = &arch.columns()[col_idx] as *const _;
            let entities = arch.entities().as_ptr();
            let scene = scene as *const Scene;
            accesses.push(QueryAccess {
                entities,
                scene,
                col,
                len: arch.len(),
                row: 0,
            });
        },
    );
    accesses
}

fn build_single_write_accesses<'a, P: WriteFetch<'a>>(
    scene: *mut Scene,
    state: &QueryFilterState,
) -> Vec<QueryMutAccess> {
    let mut accesses = Vec::new();
    let scene_ref = unsafe { &*scene };
    for_each_matching_archetype_mut(
        scene_ref,
        state,
        P::type_id(),
        &[P::type_id()],
        |scene, arch_id, col_idx| {
            let arch = scene.archetypes().get(arch_id);
            let len = arch.len();
            let col = &arch.columns()[col_idx] as *const _ as *mut _;
            let entities = arch.entities().as_ptr();
            accesses.push(QueryMutAccess {
                entities,
                col,
                scene: scene as *const Scene as *mut Scene,
                len,
                row: 0,
            });
        },
    );
    accesses
}

mod single;
mod tuples;
