use super::*;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum Collider {
    Rectangle { size: v2 },
    Cuboid { size: v3 },
    Circle { radius: f32 },
    Sphere { radius: f32 },
    Capsule { height: f32, radius: f32 },
}

impl Collider {
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self::Rectangle {
            size: v2::new(width, height),
        }
    }

    pub fn cuboid(width: f32, height: f32, depth: f32) -> Self {
        Self::Cuboid {
            size: v3::new(width, height, depth),
        }
    }

    pub fn circle(radius: f32) -> Self {
        Self::Circle { radius }
    }

    pub fn sphere(radius: f32) -> Self {
        Self::Sphere { radius }
    }

    pub fn capsule(height: f32, radius: f32) -> Self {
        Self::Capsule { height, radius }
    }
}

impl Gizmo for Collider {
    fn draw_gizmo(
        &self,
        position: v3,
        _rotation: EulerAngles,
        _scale: v3,
        buffer: &mut GizmoBuffer,
    ) {
        use comet_colors::LinearRgba;
        let color = LinearRgba::new(0.0, 1.0, 0.0, 1.0);
        match self {
            Collider::Rectangle { size } => {
                buffer.draw_rect(position, v3::new(size.x(), size.y(), 0.0), color);
            }
            Collider::Circle { radius } => {
                buffer.draw_circle(position, *radius, color);
            }
            Collider::Cuboid { size } => {
                buffer.draw_rect(position, *size, color);
            }
            Collider::Sphere { radius } => {
                buffer.draw_circle(position, *radius, color);
            }
            Collider::Capsule { height, radius } => {
                buffer.draw_rect(position, v3::new(*radius * 2.0, *height, 0.0), color);
                buffer.draw_circle(
                    v3::new(position.x(), position.y() + height * 0.5, position.z()),
                    *radius,
                    color,
                );
                buffer.draw_circle(
                    v3::new(position.x(), position.y() - height * 0.5, position.z()),
                    *radius,
                    color,
                );
            }
        }
    }
}
