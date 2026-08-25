use std::any::TypeId;

pub(crate) fn has_duplicate_type_ids(ids: &[TypeId]) -> bool {
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            if ids[i] == ids[j] {
                return true;
            }
        }
    }
    false
}
