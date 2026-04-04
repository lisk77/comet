// This is collection of basic components that are implemented out of the box
// You can use these components as is or as a reference to create your own components
// Also just as a nomenclature: bundles are a component made up of multiple components,
// so it's a collection of components bundled together (like Transform2d)
// They are intended to work with the base suite of systems provided by the engine.
use crate::math::{v2, v3, v4, m4};
use crate::{Entity, Scene};
use comet_colors::{Color, LinearRgba};
use comet_log::*;
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

pub trait Camera {
    fn get_visible_entities(&self, camera_position: &v3, scene: &Scene) -> Vec<Entity>;
    fn get_projection_matrix(&self) -> m4;
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
pub struct Camera2d {
    zoom: f32,
    dimensions: v2,
    priority: u8,
}

impl Camera2d {
    pub fn new(dimensions: v2, zoom: f32, priority: u8) -> Self {
        Self {
            dimensions,
            zoom,
            priority,
        }
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    pub fn dimensions(&self) -> v2 {
        self.dimensions
    }

    pub fn set_dimensions(&mut self, dimensions: v2) {
        self.dimensions = dimensions;
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }

    pub fn set_priority(&mut self, priority: u8) {
        self.priority = priority;
    }

    pub fn in_view_frustum(&self, camera_pos: &v3, entity_pos: &v3) -> bool {
        let left = camera_pos.x() - self.zoom;
        let right = camera_pos.x() + self.zoom;
        let bottom = camera_pos.y() - self.zoom;
        let top = camera_pos.y() + self.zoom;

        entity_pos.x() < right && entity_pos.x() > left && entity_pos.y() < top && entity_pos.y() > bottom
    }
}

impl Camera for Camera2d {
    fn get_visible_entities(&self, camera_position: &v3, scene: &Scene) -> Vec<Entity> {
        let entities = scene.entities();
        let mut visible_entities = Vec::new();
        for entity in entities {
            if let Some(ent) = entity.clone() {
                let id = ent.id();
                if let Some(transform) = scene.get_component::<Transform>(id) {
                    if self.in_view_frustum(camera_position, &transform.position()) {
                        visible_entities.push(ent);
                    }
                } else {
                    error!("Entity {} missing Transform", id.index);
                }
            }
        }
        visible_entities
    }

    fn get_projection_matrix(&self) -> m4 {
        let left = -self.dimensions.x() / 2.0;
        let right = self.dimensions.x() / 2.0;
        let bottom = -self.dimensions.y() / 2.0;
        let top = self.dimensions.y() / 2.0;

        m4::OPENGL_CONV * m4::orthographic_projection(left, right, bottom, top, 1.0, 0.0)
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