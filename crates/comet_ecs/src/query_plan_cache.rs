use std::any::TypeId;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash)]
struct SinglePlanKey {
    component: TypeId,
    with_components: Vec<TypeId>,
    without_components: Vec<TypeId>,
    with_any_components: Vec<TypeId>,
    without_any_components: Vec<TypeId>,
}

#[derive(Default)]
pub(crate) struct QueryPlanCache {
    version: usize,
    single: HashMap<SinglePlanKey, Vec<(usize, usize)>>,
}

impl QueryPlanCache {
    pub(crate) fn sync_version(&mut self, version: usize) {
        if self.version != version {
            self.version = version;
            self.single.clear();
        }
    }

    pub(crate) fn get_single_cloned(
        &self,
        component: TypeId,
        with_components: &[TypeId],
        without_components: &[TypeId],
        with_any_components: &[TypeId],
        without_any_components: &[TypeId],
    ) -> Option<Vec<(usize, usize)>> {
        self.single
            .get(&SinglePlanKey {
                component,
                with_components: with_components.to_vec(),
                without_components: without_components.to_vec(),
                with_any_components: with_any_components.to_vec(),
                without_any_components: without_any_components.to_vec(),
            })
            .cloned()
    }

    pub(crate) fn insert_single(
        &mut self,
        component: TypeId,
        with_components: &[TypeId],
        without_components: &[TypeId],
        with_any_components: &[TypeId],
        without_any_components: &[TypeId],
        matches: Vec<(usize, usize)>,
    ) {
        self.single.insert(
            SinglePlanKey {
                component,
                with_components: with_components.to_vec(),
                without_components: without_components.to_vec(),
                with_any_components: with_any_components.to_vec(),
                without_any_components: without_any_components.to_vec(),
            },
            matches,
        );
    }
}
