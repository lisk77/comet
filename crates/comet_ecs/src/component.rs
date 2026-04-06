// This is collection of basic components that are implemented out of the box
// You can use these components as is or as a reference to create your own components
// Also just as a nomenclature: bundles are a component made up of multiple components,
// so it's a collection of components bundled together (like Transform2d)
// They are intended to work with the base suite of systems provided by the engine.
use comet_gizmos::{Gizmo, GizmoBuffer};
use crate::math::{v2, v3, v4, m4};
use comet_colors::{Color, LinearRgba};
use comet_assets::{Asset, Image, ImageRef};
use component_derive::Component;

pub trait Component: Send + Sync + 'static {
    fn new() -> Self
    where
        Self: Sized + Default,
    {
        Default::default()
    }

    fn type_id() -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn type_name() -> String {
        std::any::type_name::<Self>().to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection {
    Orthographic,
    Perspective { fov: f32, near: f32, far: f32 },
    Custom { matrix: m4 },
}

impl Default for Projection {
    fn default() -> Self { Self::Orthographic }
}

#[derive(Component)]
pub struct Transform {
    position: v3,
    rotation: v3,
    scale: v3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: v3::ZERO,
            rotation: v3::ZERO,
            scale: v3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn with_position(position: v3) -> Self {
        Self {
            position,
            rotation: v3::ZERO,
            scale: v3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn with_rotation(rotation: v3) -> Self {
        Self {
            position: v3::ZERO,
            rotation,
            scale: v3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn with_scale(scale: v3) -> Self {
        Self {
            position: v3::ZERO,
            rotation: v3::ZERO,
            scale,
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

    pub fn rotation(&self) -> v3 {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: v3) {
        self.rotation = rotation;
    }

    pub fn set_rotation_x(&mut self, x: f32) {
        self.rotation.x = x;
    }

    pub fn set_rotation_y(&mut self, y: f32) {
        self.rotation.y = y;
    }

    pub fn set_rotation_z(&mut self, z: f32) {
        self.rotation.z = z;
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

#[derive(Component)]
pub enum Collider {
    Rectangle {
        size: v2,
    },
    Box {
        size: v3,
    },
    Circle {
        radius: f32,
    },
    Sphere {
        radius: f32,
    },
    Capsule {
        height: f32,
        radius: f32,
    },
}

impl Collider {
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self::Rectangle { size: v2::new(width, height) }
    }

    pub fn box_col(width: f32, height: f32, depth: f32) -> Self {
        Self::Box { size: v3::new(width, height, depth) }
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

#[derive(Component)]
pub struct Sprite {
    is_visible: bool,
    texture: ImageRef,
    draw_index: u32,
}

impl Sprite {
    pub fn new(texture: &'static str, is_visible: bool, draw_index: u32) -> Self {
        Self {
            is_visible,
            texture: ImageRef::Unresolved(texture),
            draw_index,
        }
    }

    pub fn with_texture(texture: &'static str) -> Self {
        Self {
            is_visible: true,
            texture: ImageRef::Unresolved(texture),
            draw_index: 0,
        }
    }

    pub fn with_handle(handle: Asset<Image>) -> Self {
        Self {
            is_visible: true,
            texture: ImageRef::Handle(handle),
            draw_index: 0,
        }
    }

    pub fn draw_index(&self) -> u32 {
        self.draw_index
    }

    pub fn set_draw_index(&mut self, index: u32) {
        self.draw_index = index
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn set_visibility(&mut self, is_visible: bool) {
        self.is_visible = is_visible;
    }

    pub fn texture(&self) -> ImageRef {
        self.texture
    }

    pub fn set_texture(&mut self, texture: &'static str) {
        self.texture = ImageRef::Unresolved(texture);
    }

    pub fn set_texture_asset(&mut self, texture: Asset<Image>) {
        self.texture = ImageRef::Handle(texture);
    }

    pub fn set_image_ref(&mut self, image_ref: ImageRef) {
        self.texture = image_ref;
    }
}

#[derive(Component)]
pub struct Camera {
    pub zoom: f32,
    pub priority: u8,
    pub projection: Projection,
}

impl Camera {
    pub fn new(zoom: f32, priority: u8, projection: Projection) -> Self {
        Self { zoom, priority, projection }
    }

    pub fn zoom(&self) -> f32 { self.zoom }
    pub fn set_zoom(&mut self, zoom: f32) { self.zoom = zoom; }
    pub fn priority(&self) -> u8 { self.priority }
    pub fn set_priority(&mut self, priority: u8) { self.priority = priority; }
    pub fn projection(&self) -> &Projection { &self.projection }
    pub fn set_projection(&mut self, projection: Projection) { self.projection = projection; }
}

pub struct Camera2d {
    pub transform: Transform,
    pub camera: Camera,
}

impl Camera2d {
    pub fn new(zoom: f32, priority: u8) -> Self {
        Self {
            transform: Transform::new(),
            camera: Camera::new(zoom, priority, Projection::Orthographic),
        }
    }
}

impl crate::Bundle for Camera2d {
    fn into_components(self) -> Vec<crate::prefabs::ErasedComponent> {
        vec![
            crate::prefabs::ErasedComponent::new(self.transform),
            crate::prefabs::ErasedComponent::new(self.camera),
        ]
    }
}

pub struct Camera3d {
    pub transform: Transform,
    pub camera: Camera,
}

impl Camera3d {
    pub fn new(fov: f32, near: f32, far: f32, priority: u8) -> Self {
        Self {
            transform: Transform::new(),
            camera: Camera::new(1.0, priority, Projection::Perspective { fov, near, far }),
        }
    }
}

impl crate::Bundle for Camera3d {
    fn into_components(self) -> Vec<crate::prefabs::ErasedComponent> {
        vec![
            crate::prefabs::ErasedComponent::new(self.transform),
            crate::prefabs::ErasedComponent::new(self.camera),
        ]
    }
}

#[derive(Component)]
pub struct Text {
    content: String,
    font: comet_assets::Asset<comet_assets::Font>,
    font_size: f32,
    color: v4,
    is_visible: bool,
    bounds: v2,
}

impl Text {
    pub fn new(
        content: impl Into<String>,
        font: comet_assets::Asset<comet_assets::Font>,
        font_size: f32,
        is_visible: bool,
        color: impl Color,
    ) -> Self {
        Self {
            content: content.into(),
            font,
            font_size,
            color: color.to_vec(),
            is_visible,
            bounds: v2::ZERO,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    pub fn font(&self) -> comet_assets::Asset<comet_assets::Font> {
        self.font
    }

    pub fn set_font(&mut self, font: comet_assets::Asset<comet_assets::Font>) {
        self.font = font;
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }

    pub fn color(&self) -> impl Color {
        LinearRgba::from_vec(self.color)
    }

    pub fn set_visibility(&mut self, visibility: bool) {
        self.is_visible = visibility
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn bounds(&self) -> v2 {
        self.bounds
    }

    pub fn set_bounds(&mut self, bounds: v2) {
        self.bounds = bounds
    }
}

#[derive(Component)]
pub struct Timer {
    time_stack: f32,
    interval: f32,
    done: bool,
}

impl Timer {
    pub fn set_interval(&mut self, interval: f32) {
        self.interval = interval
    }

    pub fn update_timer(&mut self, elapsed_time: f32) {
        self.time_stack += elapsed_time;
        if self.time_stack > self.interval {
            self.done = true
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn reset(&mut self) {
        self.time_stack = 0.0;
        self.done = false;
    }
}

impl Gizmo for Collider {
    fn draw_gizmo(&self, position: v3, _rotation: v3, _scale: v3, buffer: &mut GizmoBuffer) {
        use comet_colors::LinearRgba;
        let color = LinearRgba::new(0.0, 1.0, 0.0, 1.0);
        match self {
            Collider::Rectangle { size } => {
                buffer.draw_rect(position, v3::new(size.x(), size.y(), 0.0), color);
            }
            Collider::Circle { radius } => {
                buffer.draw_circle(position, *radius, color);
            }
            Collider::Box { size } => {
                buffer.draw_rect(position, *size, color);
            }
            Collider::Sphere { radius } => {
                buffer.draw_circle(position, *radius, color);
            }
            Collider::Capsule { height, radius } => {
                buffer.draw_rect(position, v3::new(*radius * 2.0, *height, 0.0), color);
                buffer.draw_circle(v3::new(position.x(), position.y() + height * 0.5, position.z()), *radius, color);
                buffer.draw_circle(v3::new(position.x(), position.y() - height * 0.5, position.z()), *radius, color);
            }
        }
    }
}