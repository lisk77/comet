mod camera;
mod collider;
mod mesh;
mod render_asset;
mod screen;
mod sprite;
mod text;
mod timer;
mod transform;

pub use camera::*;
pub use collider::*;
pub use mesh::*;
pub use render_asset::*;
pub use screen::*;
pub use sprite::*;
pub use text::*;
pub use timer::*;
pub use transform::*;

// This is collection of basic components that are implemented out of the box
// You can use these components as is or as a reference to create your own components
// Also just as a nomenclature: bundles are a component made up of multiple components,
// so it's a collection of components bundled together (like Transform2d)
// They are intended to work with the base suite of systems provided by the engine.
use crate::math::{deg, dp, m4, v2, v3, v4, Dp, EulerAngles, Px, Rad, ScreenSize, ScreenUnit};
use comet_assets::{AssetSource, Image, ImageRef};
use comet_colors::{Color, LinearRgba};
use comet_gizmos::{Gizmo, GizmoBuffer};
use comet_macros::Component;
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

#[derive(Clone, Copy)]
pub(crate) struct NeededComponent {
    pub(crate) type_id: TypeId,
    pub(crate) type_name: &'static str,
}

pub struct NeededComponents {
    components: Vec<NeededComponent>,
}

impl NeededComponents {
    pub(crate) fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn need<C: Component>(&mut self) {
        let type_id = TypeId::of::<C>();
        if self
            .components
            .iter()
            .any(|needed| needed.type_id == type_id)
        {
            return;
        }
        self.components.push(NeededComponent {
            type_id,
            type_name: std::any::type_name::<C>(),
        });
    }

    pub(crate) fn into_components(self) -> Vec<NeededComponent> {
        self.components
    }
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

    fn register_needed_components(_needs: &mut NeededComponents)
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
