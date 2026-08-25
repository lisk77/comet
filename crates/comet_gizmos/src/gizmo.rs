use crate::GizmoBuffer;
use comet_math::{v3, EulerAngles};

pub trait Gizmo {
    fn draw_gizmo(&self, position: v3, rotation: EulerAngles, scale: v3, buffer: &mut GizmoBuffer);
}
