use comet_math::v3;
use crate::GizmoBuffer;

pub trait Gizmo {
    fn draw_gizmo(&self, position: v3, rotation: v3, scale: v3, buffer: &mut GizmoBuffer);
}
