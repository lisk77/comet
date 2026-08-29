use std::any::TypeId;

use crate::Component;

pub trait Material: Component {
    fn material_type_id(&self) -> TypeId {
        self.component_type_id()
    }

    fn shader_path(&self) -> &'static str;
}
