use comet_math::v3;
use comet_colors::{Color, LinearRgba};

pub enum GizmoShape {
    Line { a: v3, b: v3, color: LinearRgba },
    Rect { position: v3, size: v3, color: LinearRgba },
    Circle { position: v3, radius: f32, color: LinearRgba },
    NGon { position: v3, radius: f32, vertices: u32, color: LinearRgba },
}

#[derive(Default)]
pub struct GizmoBuffer {
    pub shapes: Vec<GizmoShape>,
}

impl GizmoBuffer {
    pub fn new() -> Self {
        Self { shapes: Vec::new() }
    }

    pub fn draw_line(&mut self, a: v3, b: v3, color: impl Color) {
        self.shapes.push(GizmoShape::Line { a, b, color: color.to_linear() });
    }

    pub fn draw_rect(&mut self, position: v3, size: v3, color: impl Color) {
        self.shapes.push(GizmoShape::Rect { position, size, color: color.to_linear() });
    }

    pub fn draw_circle(&mut self, position: v3, radius: f32, color: impl Color) {
        self.shapes.push(GizmoShape::Circle { position, radius, color: color.to_linear() });
    }

    pub fn draw_ngon(&mut self, position: v3, radius: f32, vertices: u32, color: impl Color) {
        self.shapes.push(GizmoShape::NGon { position, radius, vertices, color: color.to_linear() });
    }

    pub fn clear(&mut self) {
        self.shapes.clear();
    }
}
