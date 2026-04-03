use comet_math::v2;
use comet_colors::{Color, LinearRgba};

pub enum GizmoShape {
    Line { a: v2, b: v2, color: LinearRgba },
    Rect { position: v2, size: v2, color: LinearRgba },
    Circle { position: v2, radius: f32, color: LinearRgba },
    NGon { position: v2, radius: f32, vertices: u32, color: LinearRgba },
}

#[derive(Default)]
pub struct GizmoBuffer {
    pub shapes: Vec<GizmoShape>,
}

impl GizmoBuffer {
    pub fn new() -> Self {
        Self { shapes: Vec::new() }
    }

    pub fn draw_line(&mut self, a: v2, b: v2, color: impl Color) {
        self.shapes.push(GizmoShape::Line { a, b, color: color.to_linear() });
    }

    pub fn draw_rect(&mut self, position: v2, size: v2, color: impl Color) {
        self.shapes.push(GizmoShape::Rect { position, size, color: color.to_linear() });
    }

    pub fn draw_circle(&mut self, position: v2, radius: f32, color: impl Color) {
        self.shapes.push(GizmoShape::Circle { position, radius, color: color.to_linear() });
    }

    pub fn draw_ngon(&mut self, position: v2, radius: f32, vertices: u32, color: impl Color) {
        self.shapes.push(GizmoShape::NGon { position, radius, vertices, color: color.to_linear() });
    }

    pub fn clear(&mut self) {
        self.shapes.clear();
    }
}
