use std::any::TypeId;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash)]
struct SinglePlanKey {
    component: TypeId,
    with_tags: Vec<TypeId>,
    without_tags: Vec<TypeId>,
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
        with_tags: &[TypeId],
        without_tags: &[TypeId],
    ) -> Option<Vec<(usize, usize)>> {
        self.single
            .get(&SinglePlanKey {
                component,
                with_tags: with_tags.to_vec(),
                without_tags: without_tags.to_vec(),
            })
            .cloned()
    }

    pub(crate) fn insert_single(
        &mut self,
        component: TypeId,
        with_tags: &[TypeId],
        without_tags: &[TypeId],
        matches: Vec<(usize, usize)>,
    ) {
        self.single.insert(
            SinglePlanKey {
                component,
                with_tags: with_tags.to_vec(),
                without_tags: without_tags.to_vec(),
            },
            matches,
        );
    }

}
