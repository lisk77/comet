use std::any::TypeId;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash)]
struct SinglePlanKey {
    component: TypeId,
    tags: Vec<TypeId>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct PairPlanKey {
    a: TypeId,
    b: TypeId,
    tags: Vec<TypeId>,
}

#[derive(Default)]
pub(crate) struct QueryPlanCache {
    version: usize,
    single: HashMap<SinglePlanKey, Vec<(usize, usize)>>,
    pair: HashMap<PairPlanKey, Vec<(usize, usize, usize)>>,
}

impl QueryPlanCache {
    pub(crate) fn sync_version(&mut self, version: usize) {
        if self.version != version {
            self.version = version;
            self.single.clear();
            self.pair.clear();
        }
    }

    pub(crate) fn get_single_cloned(
        &self,
        component: TypeId,
        tags: &[TypeId],
    ) -> Option<Vec<(usize, usize)>> {
        self.single
            .get(&SinglePlanKey {
                component,
                tags: tags.to_vec(),
            })
            .cloned()
    }

    pub(crate) fn insert_single(
        &mut self,
        component: TypeId,
        tags: &[TypeId],
        matches: Vec<(usize, usize)>,
    ) {
        self.single.insert(
            SinglePlanKey {
                component,
                tags: tags.to_vec(),
            },
            matches,
        );
    }

    pub(crate) fn get_pair_cloned(
        &self,
        a: TypeId,
        b: TypeId,
        tags: &[TypeId],
    ) -> Option<Vec<(usize, usize, usize)>> {
        self.pair
            .get(&PairPlanKey {
                a,
                b,
                tags: tags.to_vec(),
            })
            .cloned()
    }

    pub(crate) fn insert_pair(
        &mut self,
        a: TypeId,
        b: TypeId,
        tags: &[TypeId],
        matches: Vec<(usize, usize, usize)>,
    ) {
        self.pair.insert(
            PairPlanKey {
                a,
                b,
                tags: tags.to_vec(),
            },
            matches,
        );
    }
}
