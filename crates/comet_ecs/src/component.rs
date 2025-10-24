// This is collection of basic components that are implemented out of the box
// You can use these components as is or as a reference to create your own components
// Also just as a nomenclature: bundles are a component made up of multiple components,
// so it's a collection of components bundled together (like Transform2D)
// They are intended to work with the base suite of systems provided by the engine.
use crate::math::{v2, v3};
use crate::{Entity, Scene};
use comet_colors::Color as ColorTrait;
use comet_math::m4;
use component_derive::Component;

// ##################################################
// #                    BASIC                       #
// ##################################################

#[derive(Component)]
pub struct Position2D {
    position: v2,
}

#[derive(Component)]
pub struct Position3D {
    position: v3,
}

#[derive(Component)]
pub struct Rotation2D {
    theta: f32,
}

#[derive(Component)]
pub struct Rotation3D {
    theta_x: f32,
    theta_y: f32,
    theta_z: f32,
}

#[derive(Component)]
pub struct Rectangle2D {
    position: Position2D,
    size: v2,
}

#[derive(Component)]
pub struct Render2D {
    is_visible: bool,
    texture_name: &'static str,
    scale: v2,
    draw_index: u32,
}

#[derive(Component)]
pub struct Camera2D {
    zoom: f32,
    dimensions: v2,
    priority: u8,
}

#[derive(Component)]
pub struct Text {
    content: &'static str,
    font: &'static str,
    font_size: f32,
    color: Color,
    is_visible: bool,
    bounds: v2,
}

#[derive(Component)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[derive(Component)]
pub struct Timer {
    time_stack: f32,
    interval: f32,
    done: bool,
}

#[derive(Component)]
pub struct AudioSource {
    name: &'static str,
    path: Option<&'static str>,
    looped: bool,
    volume: f32,
    pitch: f32,
}

// ##################################################
// #                   BUNDLES                      #
// ##################################################

#[derive(Component)]
pub struct Transform2D {
    position: Position2D,
    rotation: Rotation2D,
}

#[derive(Component)]
pub struct Transform3D {
    position: Position3D,
    rotation: Rotation3D,
}

// ##################################################
// #                    TRAITS                      #
// ##################################################

pub trait Component: Send + Sync + PartialEq + Default + 'static {
    fn new() -> Self
    where
        Self: Sized;

    fn type_id() -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn type_name() -> String {
        std::any::type_name::<Self>().to_string()
    }
}

pub trait Collider {
    fn is_colliding(&self, other: &Self) -> bool;
}

pub trait Render {
    fn is_visible(&self) -> bool;
    fn set_visibility(&mut self, is_visible: bool);
    fn get_texture(&self) -> String;
    fn set_texture(&mut self, texture: &'static str);
}

pub trait Camera {
    fn get_visible_entities(&self, camera_position: Position2D, scene: Scene) -> Vec<Entity>;
    fn get_projection_matrix(&self) -> m4;
}

// ##################################################
// #                    IMPLS                       #
// ##################################################

impl Position2D {
    pub fn from_vec(vec: v2) -> Self {
        Self { position: vec }
    }

    pub fn as_vec(&self) -> v2 {
        self.position
    }

    pub fn x(&self) -> f32 {
        self.position.x()
    }

    pub fn y(&self) -> f32 {
        self.position.y()
    }

    pub fn set_x(&mut self, new_x: f32) {
        self.position.set_x(new_x);
    }

    pub fn set_y(&mut self, new_y: f32) {
        self.position.set_y(new_y);
    }

    pub fn set_vec(&mut self, new_pos: v2) {
        self.position = new_pos;
    }
}

impl Rotation2D {
    pub fn new(angle: f32) -> Self {
        Self { theta: angle }
    }

    pub fn angle(&self) -> f32 {
        self.theta
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.theta = angle;
    }

    pub fn rotate(&mut self, delta_angle: f32) {
        self.theta += delta_angle;
    }

    pub fn to_radians(&self) -> f32 {
        self.theta.to_radians()
    }

    pub fn to_degrees(&self) -> f32 {
        self.theta.to_degrees()
    }
}

impl Position3D {
    pub fn from_vec(vec: v3) -> Self {
        Self { position: vec }
    }

    pub fn as_vec(&self) -> v3 {
        self.position
    }

    pub fn x(&self) -> f32 {
        self.position.x()
    }

    pub fn y(&self) -> f32 {
        self.position.y()
    }

    pub fn z(&self) -> f32 {
        self.position.z()
    }

    pub fn set_x(&mut self, new_x: f32) {
        self.position.set_x(new_x);
    }

    pub fn set_y(&mut self, new_y: f32) {
        self.position.set_y(new_y);
    }

    pub fn set_z(&mut self, new_z: f32) {
        self.position.set_z(new_z);
    }

    pub fn set_vec(&mut self, new_pos: v3) {
        self.position = new_pos;
    }
}

impl Rectangle2D {
    pub fn with_size(width: f32, height: f32) -> Self {
        Self {
            position: Position2D::from_vec(v2::new(0.0, 0.0)),
            size: v2::new(width, height),
        }
    }

    pub fn position(&self) -> Position2D {
        self.position
    }

    pub fn set_position(&mut self, position: Position2D) {
        self.position = position;
    }

    pub fn size(&self) -> v2 {
        self.size
    }

    pub fn set_size(&mut self, size: v2) {
        self.size = size
    }
}

impl Collider for Rectangle2D {
    fn is_colliding(&self, other: &Self) -> bool {
        let half_size1 = self.size() * 0.5;
        let half_size2 = other.size() * 0.5;

        let dx = (self.position().x() - other.position().x()).abs();
        let dy = (self.position().y() - other.position().y()).abs();

        dx < (half_size1.x() + half_size2.x()) && dy < (half_size1.y() + half_size2.y())
    }
}

impl Render2D {
    pub fn new(texture: &'static str, is_visible: bool, scale: v2, draw_index: u32) -> Self {
        Self {
            is_visible,
            texture_name: texture,
            scale,
            draw_index,
        }
    }

    pub fn with_texture(texture: &'static str) -> Self {
        Self {
            is_visible: true,
            texture_name: texture,
            scale: v2::new(1.0, 1.0),
            draw_index: 0,
        }
    }

    pub fn scale(&self) -> v2 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: v2) {
        self.scale = scale;
    }

    pub fn draw_index(&self) -> u32 {
        self.draw_index
    }

    pub fn set_draw_index(&mut self, index: u32) {
        self.draw_index = index
    }
}

impl Render for Render2D {
    fn is_visible(&self) -> bool {
        self.is_visible
    }

    fn set_visibility(&mut self, is_visible: bool) {
        self.is_visible = is_visible;
    }

    fn get_texture(&self) -> String {
        self.texture_name.parse().unwrap()
    }

    /// Use the actual file name of the texture instead of the path
    /// e.g. "comet_icon.png" instead of "resources/textures/comet_icon.png"
    /// The resource manager will already look in the resources/textures directory
    /// This behavior will later be expanded so that it can be used with other project structures
    fn set_texture(&mut self, texture: &'static str) {
        self.texture_name = texture;
    }
}

impl Transform2D {
    pub fn position(&self) -> &Position2D {
        &self.position
    }

    pub fn position_mut(&mut self) -> &mut Position2D {
        &mut self.position
    }

    pub fn rotation(&self) -> &Rotation2D {
        &self.rotation
    }

    pub fn rotation_mut(&mut self) -> &mut Rotation2D {
        &mut self.rotation
    }

    pub fn translate(&mut self, displacement: v2) {
        let x = self.position().x() + displacement.x();
        let y = self.position().y() + displacement.y();
        self.position_mut().set_x(x);
        self.position_mut().set_y(y);
    }

    pub fn rotate(&mut self, delta_angle: f32) {
        self.rotation_mut().rotate(delta_angle);
    }

    pub fn set_rotation(&mut self, angle: f32) {
        self.rotation_mut().set_angle(angle);
    }
}

impl Transform3D {
    pub fn position(&self) -> &Position3D {
        &self.position
    }

    pub fn position_mut(&mut self) -> &mut Position3D {
        &mut self.position
    }

    pub fn rotation(&self) -> &Rotation3D {
        &self.rotation
    }

    pub fn rotation_mut(&mut self) -> &mut Rotation3D {
        &mut self.rotation
    }
}

impl Camera2D {
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

    pub fn in_view_frustum(&self, camera_pos: Position2D, entity: Position2D) -> bool {
        let left = camera_pos.x() - self.zoom;
        let right = camera_pos.x() + self.zoom;
        let bottom = camera_pos.y() - self.zoom;
        let top = camera_pos.y() + self.zoom;

        entity.x() < right && entity.x() > left && entity.y() < top && entity.y() > bottom
    }
}

impl Camera for Camera2D {
    fn get_visible_entities(&self, camera_position: Position2D, scene: Scene) -> Vec<Entity> {
        let entities = scene.entities();
        let mut visible_entities = Vec::new();
        for entity in entities {
            if self.in_view_frustum(
                camera_position,
                *scene
                    .get_component::<Transform2D>(*entity.clone().unwrap().id() as usize)
                    .unwrap()
                    .position(),
            ) {
                visible_entities.push(entity.clone().unwrap());
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

impl Text {
    pub fn new(
        content: &'static str,
        font: &'static str,
        font_size: f32,
        is_visible: bool,
        color: impl ColorTrait,
    ) -> Self {
        Self {
            content,
            font,
            font_size,
            color: Color::from_wgpu_color(color.to_wgpu()),
            is_visible,
            bounds: v2::ZERO,
        }
    }

    pub fn content(&self) -> &'static str {
        self.content
    }

    pub fn set_content(&mut self, content: &'static str) {
        self.content = content;
    }

    pub fn font(&self) -> &'static str {
        self.font
    }

    pub fn set_font(&mut self, font: &'static str) {
        self.font = font;
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn set_visibility(&mut self, visibility: bool) {
        self.is_visible = visibility
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn r(&self) -> f32 {
        self.r
    }

    pub fn set_r(&mut self, r: f32) {
        self.r = r;
    }

    pub fn g(&self) -> f32 {
        self.g
    }

    pub fn set_g(&mut self, g: f32) {
        self.g = g;
    }

    pub fn b(&self) -> f32 {
        self.b
    }

    pub fn set_b(&mut self, b: f32) {
        self.b = b;
    }

    pub fn a(&self) -> f32 {
        self.a
    }

    pub fn set_a(&mut self, a: f32) {
        self.a = a;
    }

    pub fn from_wgpu_color(color: wgpu::Color) -> Self {
        Self {
            r: color.r as f32,
            g: color.g as f32,
            b: color.b as f32,
            a: color.a as f32,
        }
    }

    pub fn to_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
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

impl AudioSource {
    pub fn new(name: &'static str, path: Option<&'static str>) -> Self {
        Self {
            name,
            path,
            looped: false,
            volume: 1.0,
            pitch: 1.0,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn path(&self) -> Option<&str> {
        self.path
    }

    pub fn looped(&self) -> bool {
        self.looped
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn set_looped(&mut self, looped: bool) {
        self.looped = looped;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
    }
}
