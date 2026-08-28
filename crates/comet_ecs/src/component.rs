// This is collection of basic components that are implemented out of the box
// You can use these components as is or as a reference to create your own components
// Also just as a nomenclature: bundles are a component made up of multiple components,
// so it's a collection of components bundled together (like Transform2d)
// They are intended to work with the base suite of systems provided by the engine.
use crate::math::{deg, dp, m4, v2, v3, v4, Dp, EulerAngles, Px, Rad, ScreenSize, ScreenUnit};
use comet_assets::{AssetSource, Image, ImageRef};
use comet_colors::{Color, LinearRgba};
use comet_gizmos::{Gizmo, GizmoBuffer};
use component_derive::Component;
use std::any::TypeId;
use std::mem::MaybeUninit;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct RequiredComponent {
    pub(crate) type_id: TypeId,
    pub(crate) register_fn: fn(&mut crate::Scene),
    pub(crate) factory: Arc<dyn Fn() -> crate::ErasedComponent + Send + Sync>,
}

pub struct RequiredComponents {
    components: Vec<RequiredComponent>,
}

impl RequiredComponents {
    pub(crate) fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn require<C: Component + Default>(&mut self) {
        self.require_with(C::default);
    }

    pub fn require_with<C: Component>(&mut self, factory: fn() -> C) {
        fn register<C: Component>(scene: &mut crate::Scene) {
            scene.ensure_component::<C>();
        }

        let type_id = TypeId::of::<C>();
        if self
            .components
            .iter()
            .any(|required| required.type_id == type_id)
        {
            return;
        }

        self.components.push(RequiredComponent {
            type_id,
            register_fn: register::<C>,
            factory: Arc::new(move || crate::ErasedComponent::new(factory())),
        });
    }

    pub(crate) fn into_components(self) -> Vec<RequiredComponent> {
        self.components
    }
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct QueryCaster {
    cast_ref: unsafe fn(*const u8, *mut ()),
    cast_mut: unsafe fn(*mut u8, *mut ()),
}

impl QueryCaster {
    pub fn new(
        cast_ref: unsafe fn(*const u8, *mut ()),
        cast_mut: unsafe fn(*mut u8, *mut ()),
    ) -> Self {
        Self { cast_ref, cast_mut }
    }

    pub(crate) unsafe fn cast_ref<T: ?Sized>(&self, value: *const u8) -> *const T {
        let mut output = MaybeUninit::<*const T>::uninit();
        unsafe { (self.cast_ref)(value, output.as_mut_ptr().cast()) };
        unsafe { output.assume_init() }
    }

    pub(crate) unsafe fn cast_mut<T: ?Sized>(&self, value: *mut u8) -> *mut T {
        let mut output = MaybeUninit::<*mut T>::uninit();
        unsafe { (self.cast_mut)(value, output.as_mut_ptr().cast()) };
        unsafe { output.assume_init() }
    }
}

#[derive(Clone)]
pub(crate) struct QueryTarget {
    pub(crate) component_type: TypeId,
    pub(crate) target_type: TypeId,
    pub(crate) caster: QueryCaster,
}

#[doc(hidden)]
pub struct QueryTargets {
    component_type: TypeId,
    targets: Vec<QueryTarget>,
}

impl QueryTargets {
    pub(crate) fn new<C: Component>() -> Self {
        Self {
            component_type: TypeId::of::<C>(),
            targets: Vec::new(),
        }
    }

    pub fn register<T: ?Sized + Component>(
        &mut self,
        cast_ref: unsafe fn(*const u8, *mut ()),
        cast_mut: unsafe fn(*mut u8, *mut ()),
    ) {
        assert!(
            self.targets
                .iter()
                .all(|target| target.target_type != TypeId::of::<T>()),
            "component registered the same query target more than once"
        );
        self.targets.push(QueryTarget {
            component_type: self.component_type,
            target_type: TypeId::of::<T>(),
            caster: QueryCaster::new(cast_ref, cast_mut),
        });
    }

    pub(crate) fn register_component<C: Component>(&mut self) {
        unsafe fn cast_ref<C: Component>(value: *const u8, output: *mut ()) {
            unsafe { output.cast::<*const C>().write(value.cast::<C>()) };
        }

        unsafe fn cast_mut<C: Component>(value: *mut u8, output: *mut ()) {
            unsafe { output.cast::<*mut C>().write(value.cast::<C>()) };
        }

        self.register::<C>(cast_ref::<C>, cast_mut::<C>);
    }

    pub(crate) fn into_targets(self) -> Vec<QueryTarget> {
        self.targets
    }
}

pub trait Component: Send + Sync + 'static {
    fn component_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn new() -> Self
    where
        Self: Sized + Default,
    {
        Default::default()
    }

    fn register_required_components(_requirements: &mut RequiredComponents)
    where
        Self: Sized,
    {
    }

    fn register_query_targets(_targets: &mut QueryTargets)
    where
        Self: Sized,
    {
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct ScreenPosition {
    anchor: Anchor,
    offset: v2,
}

impl ScreenPosition {
    pub fn new(anchor: Anchor) -> Self {
        Self {
            anchor,
            offset: v2::ZERO,
        }
    }

    pub fn with_offset(mut self, offset: v2) -> Self {
        self.offset = offset;
        self
    }

    pub fn anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }

    pub fn offset(&self) -> v2 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: v2) {
        self.offset = offset;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResolutionScaling {
    /// Fits the virtual canvas height to the output and lets visible width follow its aspect ratio.
    FitVertical,
    /// Fits the virtual canvas width to the output and lets visible height follow its aspect ratio.
    FitHorizontal,
    /// Shows the entire virtual canvas, adding letterboxing or pillarboxing when necessary.
    Fit,
    /// Fills the output while cropping virtual canvas content on one axis when necessary.
    Fill,
    /// Fills the output by revealing additional world beyond the virtual canvas on one axis.
    #[default]
    Expand,
    /// Maps the complete virtual canvas to the output without preserving its aspect ratio.
    Stretch,
}

/// A camera's output rectangle in physical render-target pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraViewport {
    x: Px,
    y: Px,
    width: Px,
    height: Px,
}

impl CameraViewport {
    pub fn new(x: Px, y: Px, width: Px, height: Px) -> Self {
        assert!(
            x.pixels().is_finite()
                && y.pixels().is_finite()
                && width.pixels().is_finite()
                && height.pixels().is_finite()
                && x.pixels() >= 0.0
                && y.pixels() >= 0.0
                && width.pixels() > 0.0
                && height.pixels() > 0.0,
            "camera viewport must have a finite, non-negative origin and positive dimensions"
        );
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn x(&self) -> Px {
        self.x
    }
    pub fn y(&self) -> Px {
        self.y
    }
    pub fn width(&self) -> Px {
        self.width
    }
    pub fn height(&self) -> Px {
        self.height
    }
}

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

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Sprite {
    is_visible: bool,
    texture: ImageRef,
    draw_index: u32,
}

impl Sprite {
    pub fn with_texture(texture: impl Into<AssetSource<Image>>) -> Self {
        Self {
            is_visible: true,
            texture: texture.into().into(),
            draw_index: 0,
        }
    }

    pub fn draw_index(&self) -> u32 {
        self.draw_index
    }

    pub fn with_draw_index(mut self, index: u32) -> Self {
        self.draw_index = index;
        self
    }

    pub fn set_draw_index(&mut self, index: u32) {
        self.draw_index = index
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn with_visibility(mut self, is_visible: bool) -> Self {
        self.is_visible = is_visible;
        self
    }

    pub fn set_visibility(&mut self, is_visible: bool) {
        self.is_visible = is_visible;
    }

    pub fn texture(&self) -> ImageRef {
        self.texture.clone()
    }

    pub fn set_texture(&mut self, texture: impl Into<AssetSource<Image>>) {
        self.texture = texture.into().into();
    }

    pub fn set_image_ref(&mut self, image_ref: ImageRef) {
        self.texture = image_ref;
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

#[derive(Debug, Clone, PartialEq)]
pub struct Screen {
    virtual_resolution: Option<ScreenSize>,
    resolution_scaling: ResolutionScaling,
    viewport: Option<CameraViewport>,
}

impl Component for Screen {}

impl Default for Screen {
    fn default() -> Self {
        Self {
            virtual_resolution: None,
            resolution_scaling: ResolutionScaling::default(),
            viewport: None,
        }
    }
}

impl Screen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_virtual_resolution(
        mut self,
        width: impl Into<ScreenUnit>,
        height: impl Into<ScreenUnit>,
    ) -> Self {
        self.set_virtual_resolution(width, height);
        self
    }

    pub fn with_resolution_scaling(mut self, scaling: ResolutionScaling) -> Self {
        self.resolution_scaling = scaling;
        self
    }

    pub fn with_viewport(mut self, viewport: CameraViewport) -> Self {
        self.viewport = Some(viewport);
        self
    }

    pub fn virtual_resolution(&self) -> Option<ScreenSize> {
        self.virtual_resolution
    }

    pub fn set_virtual_resolution(
        &mut self,
        width: impl Into<ScreenUnit>,
        height: impl Into<ScreenUnit>,
    ) {
        let resolution = ScreenSize::new(width, height);
        let resolved = resolution.resolve(1.0);
        assert!(
            resolved.x().is_finite()
                && resolved.y().is_finite()
                && resolved.x() > 0.0
                && resolved.y() > 0.0,
            "virtual resolution dimensions must be finite and greater than zero"
        );
        self.virtual_resolution = Some(resolution);
    }

    pub fn clear_virtual_resolution(&mut self) {
        self.virtual_resolution = None;
    }

    pub fn resolution_scaling(&self) -> ResolutionScaling {
        self.resolution_scaling
    }

    pub fn set_resolution_scaling(&mut self, scaling: ResolutionScaling) {
        self.resolution_scaling = scaling;
    }

    pub fn viewport(&self) -> Option<CameraViewport> {
        self.viewport
    }

    pub fn set_viewport(&mut self, viewport: CameraViewport) {
        self.viewport = Some(viewport);
    }

    pub fn clear_viewport(&mut self) {
        self.viewport = None;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextJustification {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct TextLayout {
    anchor: Anchor,
    justification: TextJustification,
}

impl TextLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_justification(mut self, justification: TextJustification) -> Self {
        self.justification = justification;
        self
    }

    pub fn anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }

    pub fn justification(&self) -> TextJustification {
        self.justification
    }

    pub fn set_justification(&mut self, justification: TextJustification) {
        self.justification = justification;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextSize {
    Screen(ScreenUnit),
    World(f32),
}

impl Default for TextSize {
    fn default() -> Self {
        Self::Screen(dp(16.0).into())
    }
}

impl TextSize {
    fn assert_valid(self) {
        let value = match self {
            Self::Screen(ScreenUnit::Px(size)) => size.pixels(),
            Self::Screen(ScreenUnit::Dp(size)) => size.display_points(),
            Self::World(size) => size,
        };
        assert!(
            value.is_finite() && value > 0.0,
            "text size must be finite and greater than zero"
        );
    }
}

impl From<Px> for TextSize {
    fn from(size: Px) -> Self {
        Self::Screen(size.into())
    }
}

impl From<Dp> for TextSize {
    fn from(size: Dp) -> Self {
        Self::Screen(size.into())
    }
}

impl From<ScreenUnit> for TextSize {
    fn from(size: ScreenUnit) -> Self {
        Self::Screen(size)
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Text {
    content: String,
    font: comet_assets::Asset<comet_assets::Font>,
    font_size: TextSize,
    color: v4,
    is_visible: bool,
    bounds: v2,
}

impl Text {
    pub fn new(content: impl Into<String>, font: comet_assets::Asset<comet_assets::Font>) -> Self {
        Self {
            content: content.into(),
            font,
            font_size: TextSize::default(),
            color: LinearRgba::new(1.0, 1.0, 1.0, 1.0).to_vec(),
            is_visible: true,
            bounds: v2::ZERO,
        }
    }

    pub fn with_font_size(mut self, font_size: impl Into<TextSize>) -> Self {
        self.set_font_size(font_size);
        self
    }

    pub fn with_world_font_size(mut self, font_size: f32) -> Self {
        self.set_world_font_size(font_size);
        self
    }

    pub fn with_color(mut self, color: impl Color) -> Self {
        self.color = color.to_vec();
        self
    }

    pub fn with_visibility(mut self, is_visible: bool) -> Self {
        self.is_visible = is_visible;
        self
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

    pub fn font_size(&self) -> TextSize {
        self.font_size
    }

    pub fn set_font_size(&mut self, font_size: impl Into<TextSize>) {
        let font_size = font_size.into();
        font_size.assert_valid();
        self.font_size = font_size;
    }

    pub fn set_world_font_size(&mut self, font_size: f32) {
        assert!(
            font_size.is_finite() && font_size > 0.0,
            "world font size must be finite and greater than zero"
        );
        self.font_size = TextSize::World(font_size);
    }

    pub fn color(&self) -> impl Color {
        LinearRgba::from_vec(self.color)
    }

    pub fn set_color(&mut self, color: impl Color) {
        self.color = color.to_vec();
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
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
