use comet_ecs::Transform;
use crate::GizmoBuffer;

pub trait Gizmo {
    fn draw_gizmo(&self, transform: &Transform, gizmos: &mut GizmoBuffer);
}
