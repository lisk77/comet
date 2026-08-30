use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    position: v3,
    rotation: EulerAngles,
    scale: v3,
}

impl Component for Transform {}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: v3::ZERO,
            rotation: EulerAngles::ZERO,
            scale: v3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn at(position: v3) -> Self {
        Self {
            position,
            ..Self::default()
        }
    }

    pub fn rotated(rotation: EulerAngles) -> Self {
        Self {
            rotation,
            ..Self::default()
        }
    }

    pub fn scaled(scale: v3) -> Self {
        Self {
            scale,
            ..Self::default()
        }
    }

    pub fn position(&self) -> v3 {
        self.position
    }

    pub fn set_position(&mut self, position: v3) {
        self.position = position;
    }

    pub fn set_x(&mut self, x: f32) {
        self.position.x = x;
    }

    pub fn set_y(&mut self, y: f32) {
        self.position.y = y;
    }

    pub fn set_z(&mut self, z: f32) {
        self.position.z = z;
    }

    pub fn rotation(&self) -> EulerAngles {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: EulerAngles) {
        self.rotation = rotation;
    }

    pub fn set_rotation_x(&mut self, angle: impl Into<Rad>) {
        self.rotation.set_x(angle);
    }

    pub fn set_rotation_y(&mut self, angle: impl Into<Rad>) {
        self.rotation.set_y(angle);
    }

    pub fn set_rotation_z(&mut self, angle: impl Into<Rad>) {
        self.rotation.set_z(angle);
    }

    pub fn scale(&self) -> v3 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: v3) {
        self.scale = scale;
    }

    pub fn set_scale_x(&mut self, x: f32) {
        self.scale.x = x;
    }

    pub fn set_scale_y(&mut self, y: f32) {
        self.scale.y = y;
    }

    pub fn set_scale_z(&mut self, z: f32) {
        self.scale.z = z;
    }

    pub fn translate(&mut self, translation: v3) {
        self.position += translation;
    }
}
