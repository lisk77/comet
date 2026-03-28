use comet_colors::{Color as ColorTrait, LinearRgba};
use comet_input::keyboard::Key;
use comet_log::*;
use crate::{
    module::Module,
    renderer::{Renderer, RendererHandle},
};
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use winit::dpi::LogicalSize;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Icon, Window},
};
use winit_input_helper::WinitInputHelper as InputManager;

const MAX_TICK_STEPS: usize = 10;

/// The `App` struct represents the common interface for many different components of the game engine.
/// It provides a unified interface for managing the application's state, input, and modules.
pub struct App {
    title: String,
    icon: Option<Icon>,
    size: Option<LogicalSize<u32>>,
    clear_color: Option<LinearRgba>,
    input_manager: Arc<Mutex<InputManager>>,
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
    /// Creates a new `App` instance.
    pub fn new() -> Self {
        Self {
            title: "Untitled".to_string(),
            icon: None,
            size: None,
            clear_color: None,
            input_manager: Arc::new(Mutex::new(InputManager::new())),
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

    /// Allows to set the title of the `App` instance.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Allows to set the icon of the `App` instance.
    pub fn with_icon(mut self, path: impl AsRef<str>) -> Self {
        self.icon = Self::load_icon(&crate::asset_path::resolve_asset_path(path));
        self
    }

    /// Allows to set the size of the `App` instance.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = Some(LogicalSize::new(width, height));
        self
    }

    /// Allows to set the clear color of the `App` instance.
    pub fn with_clear_color(mut self, clear_color: impl ColorTrait) -> Self {
        self.clear_color = Some(clear_color.to_linear());
        self
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

    /// Returns a reference to the module of type `M`. Panics if not loaded.
    pub fn get_module<M: 'static>(&self) -> &M {
        self.modules
            .get(&TypeId::of::<M>())
            .and_then(|m| (m.as_ref() as &dyn Any).downcast_ref::<M>())
            .unwrap_or_else(|| panic!("module `{}` is not loaded", type_name::<M>()))
    }

    /// Returns a mutable reference to the module of type `M`. Panics if not loaded.
    pub fn get_module_mut<M: 'static>(&mut self) -> &mut M {
        self.modules
            .get_mut(&TypeId::of::<M>())
            .and_then(|m| (m.as_mut() as &mut dyn Any).downcast_mut::<M>())
            .unwrap_or_else(|| panic!("module `{}` is not loaded", type_name::<M>()))
    }

    /// Returns whether a module of type `M` has been added.
    pub fn has_module<M: 'static>(&self) -> bool {
        self.modules.contains_key(&TypeId::of::<M>())
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
            .unwrap_or_else(|| panic!("context `{}` not found", type_name::<T>()))
    }

    /// Returns a mutable reference to the context of type `T`. Panics if not present.
    pub fn context_mut<T: Any + Send + 'static>(&mut self) -> &mut T {
        self.contexts
            .get_mut(&TypeId::of::<T>())
            .and_then(|c| c.downcast_mut::<T>())
            .unwrap_or_else(|| panic!("context `{}` not found", type_name::<T>()))
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

    /// Registers a system that runs every tick in deterministic order.
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

    fn load_icon(path: &std::path::Path) -> Option<Icon> {
        let image = match image::open(path) {
            Ok(image) => image,
            Err(_) => {
                error!("Failed loading icon {}", path.display());
                return None;
            }
        };
        let rgba_image = image.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        Some(Icon::from_rgba(rgba_image.into_raw(), width, height).unwrap())
    }

    /// Stops the event loop and with that quits the `App`.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Returns the fixed delta time set by the `App`.
    pub fn dt(&self) -> f32 {
        self.update_timer
    }

    /// Returns the last frame time as computed by the renderer.
    pub fn frame_dt(&self) -> f32 {
        self.delta_time
    }

    /// Sets the amount of times the `App` game logic is updated per second.
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

    /// Retrieves a reference to the `InputManager`.
    pub fn input_manager(&self) -> std::sync::MutexGuard<'_, InputManager> {
        self.input_manager.lock().unwrap()
    }

    /// Checks if a key is currently pressed.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.input_manager.lock().unwrap().key_pressed(key)
    }

    /// Checks if a key is currently held.
    pub fn key_held(&self, key: Key) -> bool {
        self.input_manager.lock().unwrap().key_held(key)
    }

    /// Checks if a key was released this frame.
    pub fn key_released(&self, key: Key) -> bool {
        self.input_manager.lock().unwrap().key_released(key)
    }

    fn create_window(
        app_title: String,
        app_icon: &Option<Icon>,
        window_size: &Option<LogicalSize<u32>>,
        event_loop: &EventLoop<()>,
    ) -> Window {
        let winit_window = winit::window::WindowBuilder::new().with_title(app_title);

        let winit_window = if let Some(icon) = app_icon.clone() {
            winit_window.with_window_icon(Some(icon))
        } else {
            winit_window
        };

        let winit_window = if let Some(size) = window_size.clone() {
            winit_window.with_inner_size(size)
        } else {
            winit_window
        };

        winit_window.build(event_loop).unwrap()
    }

    /// Starts the `App` without a window or renderer.
    pub fn run_headless(
        mut self,
        setup: fn(&mut App),
        update: fn(&mut App, f32),
    ) {
        let title = self.title.clone();
        info!("Starting up {} (headless)!", title);
        setup(&mut self);

        let mut time_stack = 0.0f32;
        let mut last_tick = std::time::Instant::now();

        loop {
            self.run_tick_cycle(&mut last_tick, &mut time_stack, update);

            if self.should_quit {
                break;
            }

            Self::sleep_until_next_tick(self.update_timer, last_tick);
        }

        info!("Shutting down {}!", title);
    }

    /// Starts the `App` event loop.
    pub fn run<R: Renderer>(
        self,
        setup: fn(&mut App, &mut R::Handle),
        update: fn(&mut App, &mut R::Handle, f32),
    ) where
        R::Handle: 'static,
    {
        let title = self.title.clone();
        info!("Starting up {}!", title);

        let input_manager = self.input_manager.clone();
        let update_timer = self.update_timer;

        let (cmd_tx, cmd_rx) = flume::unbounded();
        let (evt_tx, evt_rx) = flume::unbounded();

        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(Self::create_window(title.clone(), &self.icon, &self.size, &event_loop));
        let mut renderer = R::new(window, self.clear_color, evt_tx);
        renderer.init_assets(&self);
        info!("Using Renderer {}", type_name::<R>());

        let quit_flag = Arc::new(AtomicBool::new(false));
        let logic_thread = std::thread::Builder::new()
            .name("logic".to_string())
            .spawn({
                let quit = quit_flag.clone();
                move || self.run_logic_thread::<R>(setup, update, quit, cmd_tx, evt_rx)
            })
            .unwrap();

        info!("Starting event loop!");
        Self::run_event_loop::<R>(event_loop, renderer, input_manager, quit_flag, cmd_rx, update_timer);

        logic_thread.join().ok();
        info!("Shutting down {}!", title);
    }

    fn run_logic_thread<R: Renderer>(
        mut self,
        setup: fn(&mut App, &mut R::Handle),
        update: fn(&mut App, &mut R::Handle, f32),
        quit_flag: Arc<AtomicBool>,
        cmd_tx: flume::Sender<<R::Handle as RendererHandle>::Command>,
        evt_rx: flume::Receiver<<R::Handle as RendererHandle>::Event>,
    ) where
        R::Handle: 'static,
    {
        let mut handle = R::Handle::new(cmd_tx, evt_rx);
        info!("Setting up!");
        setup(&mut self, &mut handle);

        let mut time_stack = 0.0f32;
        let mut last_tick = std::time::Instant::now();

        while !quit_flag.load(Ordering::Relaxed) {
            self.run_tick_cycle(&mut last_tick, &mut time_stack, |app, dt| update(app, &mut handle, dt));

            if self.should_quit {
                quit_flag.store(true, Ordering::Relaxed);
                break;
            }

            Self::sleep_until_next_tick(self.update_timer, last_tick);
        }
    }

    fn run_tick_cycle(
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

    fn run_event_loop<R: Renderer>(
        event_loop: EventLoop<()>,
        mut renderer: R,
        input_manager: Arc<Mutex<InputManager>>,
        quit_flag: Arc<AtomicBool>,
        cmd_rx: flume::Receiver<<R::Handle as RendererHandle>::Command>,
        update_timer: f32,
    ) {
        let mut window_occluded = false;

        event_loop
            .run(|event, elwt| {
                if quit_flag.load(Ordering::Relaxed) {
                    elwt.exit();
                    return;
                }
                if let Ok(mut m) = input_manager.lock() {
                    m.update(&event);
                }
                match event {
                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::CloseRequested => {
                            quit_flag.store(true, Ordering::Relaxed);
                            elwt.exit();
                        }
                        WindowEvent::Occluded(occluded) => {
                            window_occluded = *occluded;
                        }
                        WindowEvent::Resized(size) => renderer.resize(*size),
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            renderer.set_scale_factor(*scale_factor);
                        }
                        WindowEvent::RedrawRequested => {
                            while let Ok(cmd) = cmd_rx.try_recv() {
                                renderer.apply_command(cmd);
                            }
                            if !window_occluded {
                                if Self::handle_render(&mut renderer) {
                                    elwt.exit();
                                }
                            }
                        }
                        _ => {}
                    },
                    Event::AboutToWait => {
                        if window_occluded {
                            while cmd_rx.try_recv().is_ok() {}
                            elwt.set_control_flow(ControlFlow::Wait);
                        } else {
                            while let Ok(cmd) = cmd_rx.try_recv() {
                                renderer.apply_command(cmd);
                            }
                            renderer.window().request_redraw();
                            if update_timer.is_finite() {
                                let next = std::time::Instant::now()
                                    + std::time::Duration::from_secs_f32(update_timer);
                                elwt.set_control_flow(ControlFlow::WaitUntil(next));
                            } else {
                                elwt.set_control_flow(ControlFlow::Wait);
                            }
                        }
                    }
                    _ => {}
                }
            })
            .unwrap();
    }

    fn handle_render<R: Renderer>(renderer: &mut R) -> bool {
        match renderer.render() {
            Ok(_) => false,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let size = renderer.size();
                renderer.resize(size);
                false
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                error!("Out of memory!");
                true
            }
            Err(wgpu::SurfaceError::Timeout) => {
                warn!("Surface timeout - skipping frame");
                false
            }
        }
    }

    fn sleep_until_next_tick(update_timer: f32, last_tick: std::time::Instant) {
        if update_timer.is_finite() && update_timer > 0.0 {
            let target = std::time::Duration::from_secs_f32(update_timer);
            let elapsed = last_tick.elapsed();
            if elapsed < target {
                std::thread::sleep(target - elapsed);
            }
        } else {
            std::thread::yield_now();
        }
    }
}
