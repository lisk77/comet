use super::*;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveProjection {
    vertical_fov: Rad,
    near_plane: f32,
    far_plane: f32,
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            vertical_fov: deg(60.0).into(),
            near_plane: 0.1,
            far_plane: 1_000.0,
        }
    }
}

impl PerspectiveProjection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_vertical_fov(mut self, vertical_fov: impl Into<Rad>) -> Self {
        self.set_vertical_fov(vertical_fov);
        self
    }

    pub fn with_clipping_planes(mut self, near_plane: f32, far_plane: f32) -> Self {
        self.set_clipping_planes(near_plane, far_plane);
        self
    }

    pub fn vertical_fov(&self) -> Rad {
        self.vertical_fov
    }

    pub fn set_vertical_fov(&mut self, vertical_fov: impl Into<Rad>) {
        let vertical_fov = vertical_fov.into();
        assert!(
            vertical_fov.radians().is_finite()
                && vertical_fov.radians() > 0.0
                && vertical_fov.radians() < std::f32::consts::PI,
            "vertical fov must be finite and between zero and pi radians"
        );
        self.vertical_fov = vertical_fov;
    }

    pub fn near_plane(&self) -> f32 {
        self.near_plane
    }

    pub fn far_plane(&self) -> f32 {
        self.far_plane
    }

    pub fn set_clipping_planes(&mut self, near_plane: f32, far_plane: f32) {
        assert!(
            near_plane.is_finite()
                && far_plane.is_finite()
                && near_plane > 0.0
                && far_plane > near_plane,
            "perspective clipping planes must be finite, with 0 < near < far"
        );
        self.near_plane = near_plane;
        self.far_plane = far_plane;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrthographicProjection {
    visible_height: f32,
    near_plane: f32,
    far_plane: f32,
    magnification: f32,
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        Self {
            visible_height: 1.0,
            near_plane: -1_000.0,
            far_plane: 1_000.0,
            magnification: 1.0,
        }
    }
}

impl OrthographicProjection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_visible_height(mut self, visible_height: f32) -> Self {
        self.set_visible_height(visible_height);
        self
    }

    pub fn with_clipping_planes(mut self, near_plane: f32, far_plane: f32) -> Self {
        self.set_clipping_planes(near_plane, far_plane);
        self
    }

    pub fn with_magnification(mut self, magnification: f32) -> Self {
        self.set_magnification(magnification);
        self
    }

    pub fn visible_height(&self) -> f32 {
        self.visible_height
    }

    pub fn set_visible_height(&mut self, visible_height: f32) {
        assert!(
            visible_height.is_finite() && visible_height > 0.0,
            "orthographic visible height must be finite and greater than zero"
        );
        self.visible_height = visible_height;
    }

    pub fn near_plane(&self) -> f32 {
        self.near_plane
    }

    pub fn far_plane(&self) -> f32 {
        self.far_plane
    }

    pub fn set_clipping_planes(&mut self, near_plane: f32, far_plane: f32) {
        assert!(
            near_plane.is_finite() && far_plane.is_finite() && far_plane > near_plane,
            "orthographic clipping planes must be finite, with near < far"
        );
        self.near_plane = near_plane;
        self.far_plane = far_plane;
    }

    pub fn magnification(&self) -> f32 {
        self.magnification
    }

    pub fn set_magnification(&mut self, magnification: f32) {
        assert!(
            magnification.is_finite() && magnification > 0.0,
            "camera magnification must be finite and greater than zero"
        );
        self.magnification = magnification;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection {
    Orthographic(OrthographicProjection),
    Perspective(PerspectiveProjection),
    Custom(m4),
}

impl Component for Projection {}

impl Default for Projection {
    fn default() -> Self {
        Self::Orthographic(OrthographicProjection::default())
    }
}

impl From<PerspectiveProjection> for Projection {
    fn from(projection: PerspectiveProjection) -> Self {
        Self::Perspective(projection)
    }
}

impl From<OrthographicProjection> for Projection {
    fn from(projection: OrthographicProjection) -> Self {
        Self::Orthographic(projection)
    }
}

impl From<m4> for Projection {
    fn from(matrix: m4) -> Self {
        Self::Custom(matrix)
    }
}

impl Projection {
    pub fn magnification(&self) -> f32 {
        match self {
            Self::Orthographic(projection) => projection.magnification(),
            _ => 1.0,
        }
    }

    pub fn set_magnification(&mut self, magnification: f32) {
        match self {
            Self::Orthographic(projection) => projection.set_magnification(magnification),
            _ => panic!("magnification is only available for orthographic projections"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Camera {
    enabled: bool,
    priority: i32,
}

impl Component for Camera {}

impl Default for Camera {
    fn default() -> Self {
        Self {
            enabled: true,
            priority: 0,
        }
    }
}

impl Camera {
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
    }
}

fn default_2d_projection() -> Projection {
    Projection::Orthographic(OrthographicProjection::default())
}

fn default_3d_projection() -> Projection {
    Projection::Perspective(PerspectiveProjection::default())
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[require(
    Transform,
    Camera,
    Projection = default_2d_projection,
    Screen,
)]
pub struct Camera2d;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[require(
    Transform,
    Camera,
    Projection = default_3d_projection,
    Screen,
)]
pub struct Camera3d;
