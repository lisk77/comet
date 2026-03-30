use crate::module::Module;
use comet_log::fatal;
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;

const MAX_TICK_STEPS: usize = 10;

pub struct App {
    delta_time: f32,
    update_timer: f32,
    modules: HashMap<TypeId, Box<dyn Any + Send>>,
    contexts: HashMap<TypeId, Box<dyn Any + Send>>,
    should_quit: bool,
    tick_systems: Vec<fn(&mut App, f32)>,
    pending_tick_add: Vec<fn(&mut App, f32)>,
    pending_tick_remove: Vec<fn(&mut App, f32)>,
    pre_tick_hooks: Vec<fn(&mut App)>,
    post_tick_hooks: Vec<fn(&mut App)>,
}

impl App {
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
            update_timer: 0.0166667,
            modules: HashMap::new(),
            contexts: HashMap::new(),
            should_quit: false,
            tick_systems: Vec::new(),
            pending_tick_add: Vec::new(),
            pending_tick_remove: Vec::new(),
            pre_tick_hooks: Vec::new(),
            post_tick_hooks: Vec::new(),
        }
    }

    /// Adds multiple modules at once.
    pub fn with_modules<T: crate::module_tuple::ModuleTuple>(self, modules: T) -> Self {
        modules.add_to(self)
    }

    /// Adds a module, calling its `build` method to register systems/components.
    pub fn with_module<M: Module>(mut self, mut module: M) -> Self {
        M::dependencies(&mut self);
        module.build(&mut self);
        self.modules.insert(TypeId::of::<M>(), Box::new(module));
        self
    }

    /// Adds a module at runtime (e.g. inside setup/update).
    pub fn add_module<M: Module>(&mut self, mut module: M) {
        M::dependencies(self);
        module.build(self);
        self.modules.insert(TypeId::of::<M>(), Box::new(module));
    }

    /// Re-inserts a previously taken module without calling `dependencies` or `build` again.
    pub fn reinsert_module<M: Module>(&mut self, module: M) {
        self.modules.insert(TypeId::of::<M>(), Box::new(module));
    }

    /// Returns a reference to the module of type `M`. Panics if not loaded.
    pub fn get_module<M: 'static>(&self) -> &M {
        self.modules
            .get(&TypeId::of::<M>())
            .and_then(|m| (m.as_ref() as &dyn Any).downcast_ref::<M>())
            .unwrap_or_else(|| fatal!("Module {} is not loaded", type_name::<M>()))
    }

    /// Returns a mutable reference to the module of type `M`. Panics if not loaded.
    pub fn get_module_mut<M: 'static>(&mut self) -> &mut M {
        self.modules
            .get_mut(&TypeId::of::<M>())
            .and_then(|m| (m.as_mut() as &mut dyn Any).downcast_mut::<M>())
            .unwrap_or_else(|| fatal!("Module {} is not loaded", type_name::<M>()))
    }

    /// Returns whether a module of type `M` has been added.
    pub fn has_module<M: 'static>(&self) -> bool {
        self.modules.contains_key(&TypeId::of::<M>())
    }

    /// Removes and returns the module of type `M`, if present.
    pub fn take_module<M: 'static>(&mut self) -> Option<M> {
        self.modules
            .remove(&TypeId::of::<M>())
            .and_then(|b| b.downcast::<M>().ok())
            .map(|b| *b)
    }

    /// Inserts a context value, replacing any previous value of the same type.
    pub fn add_context<T: Any + Send + 'static>(&mut self, ctx: T) {
        self.contexts.insert(TypeId::of::<T>(), Box::new(ctx));
    }

    /// Returns a reference to the context of type `T`. Panics if not present.
    pub fn context<T: Any + Send + 'static>(&self) -> &T {
        self.contexts
            .get(&TypeId::of::<T>())
            .and_then(|c| c.downcast_ref::<T>())
            .unwrap_or_else(|| fatal!("Context {} not found", type_name::<T>()))
    }

    /// Returns a mutable reference to the context of type `T`. Panics if not present.
    pub fn context_mut<T: Any + Send + 'static>(&mut self) -> &mut T {
        self.contexts
            .get_mut(&TypeId::of::<T>())
            .and_then(|c| c.downcast_mut::<T>())
            .unwrap_or_else(|| fatal!("Context {} not found", type_name::<T>()))
    }

    /// Returns a reference to the context of type `T`, or `None` if not present.
    pub fn try_get_context<T: Any + Send + 'static>(&self) -> Option<&T> {
        self.contexts.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    /// Returns a mutable reference to the context of type `T`, or `None` if not present.
    pub fn try_get_context_mut<T: Any + Send + 'static>(&mut self) -> Option<&mut T> {
        self.contexts.get_mut(&TypeId::of::<T>())?.downcast_mut::<T>()
    }

    /// Returns whether a context of type `T` has been added.
    pub fn has_context<T: Any + 'static>(&self) -> bool {
        self.contexts.contains_key(&TypeId::of::<T>())
    }

    /// Registers a system that runs every tick.
    pub fn add_tick_system(&mut self, system: fn(&mut App, f32)) {
        self.pending_tick_add.push(system);
    }

    /// Removes a tick system if present.
    pub fn remove_tick_system(&mut self, system: fn(&mut App, f32)) -> bool {
        self.pending_tick_remove.push(system);
        true
    }

    /// Registers a hook that runs before tick systems each tick.
    pub fn add_pre_tick_hook(&mut self, hook: fn(&mut App)) {
        self.pre_tick_hooks.push(hook);
    }

    /// Registers a hook that runs after tick systems and update each tick.
    pub fn add_post_tick_hook(&mut self, hook: fn(&mut App)) {
        self.post_tick_hooks.push(hook);
    }

    fn apply_tick_system_changes(&mut self) {
        if !self.pending_tick_remove.is_empty() {
            for remove in self.pending_tick_remove.drain(..) {
                if let Some(pos) = self
                    .tick_systems
                    .iter()
                    .position(|s| std::ptr::fn_addr_eq(*s, remove))
                {
                    self.tick_systems.remove(pos);
                }
            }
        }

        if !self.pending_tick_add.is_empty() {
            for system in self.pending_tick_add.drain(..) {
                if !self
                    .tick_systems
                    .iter()
                    .any(|s| std::ptr::fn_addr_eq(*s, system))
                {
                    self.tick_systems.push(system);
                }
            }
        }
    }

    /// Stops the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Returns true if `quit()` has been called.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Returns the fixed tick interval (1 / update_rate).
    pub fn dt(&self) -> f32 {
        self.update_timer
    }

    /// Returns the last measured frame delta time.
    pub fn frame_dt(&self) -> f32 {
        self.delta_time
    }

    /// Sets the number of logic ticks per second. Pass 0 for uncapped (variable timestep).
    pub fn set_update_rate(&mut self, update_rate: u32) {
        if update_rate == 0 {
            self.update_timer = f32::INFINITY;
            return;
        }
        self.update_timer = 1.0 / update_rate as f32;
    }

    fn run_pre_tick_hooks(&mut self) {
        let mut i = 0;
        while i < self.pre_tick_hooks.len() {
            let hook = self.pre_tick_hooks[i];
            hook(self);
            i += 1;
        }
    }

    fn run_post_tick_hooks(&mut self) {
        let mut i = 0;
        while i < self.post_tick_hooks.len() {
            let hook = self.post_tick_hooks[i];
            hook(self);
            i += 1;
        }
    }

    pub fn run_tick_cycle(
        &mut self,
        last_tick: &mut std::time::Instant,
        time_stack: &mut f32,
        mut update: impl FnMut(&mut App, f32),
    ) {
        let now = std::time::Instant::now();
        let frame_dt = now.duration_since(*last_tick).as_secs_f32();
        *last_tick = now;
        self.delta_time = frame_dt;

        if self.update_timer.is_finite() {
            *time_stack += frame_dt;
            let mut steps = 0;
            while *time_stack > self.update_timer && steps < MAX_TICK_STEPS {
                let step = self.update_timer;
                self.run_single_tick(step, &mut update);
                *time_stack -= self.update_timer;
                steps += 1;
            }
        } else {
            self.run_single_tick(frame_dt, &mut update);
        }
    }

    fn run_single_tick(&mut self, dt: f32, update: &mut impl FnMut(&mut App, f32)) {
        self.run_pre_tick_hooks();
        self.apply_tick_system_changes();
        let mut i = 0;
        while i < self.tick_systems.len() {
            let system = self.tick_systems[i];
            system(self, dt);
            i += 1;
        }
        update(self, dt);
        self.run_post_tick_hooks();
    }

}
