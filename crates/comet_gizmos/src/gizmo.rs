use comet_ecs::Transform2D;
use crate::GizmoBuffer;

pub trait Gizmo {
    fn draw_gizmo(&self, transform: &Transform2D, gizmos: &mut GizmoBuffer);
}
