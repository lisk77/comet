use comet_colors::{Color as ColorTrait, LinearRgba};
use comet_ecs::{
    Camera2D, Component, Entity, Render2D, Scene, Text, Transform2D, Transform3D,
};
use comet_input::keyboard::Key;
use comet_log::*;
use comet_renderer::renderer::{Renderer, RendererHandle};
use comet_sound::*;
use std::any::{type_name, Any, TypeId};
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

/// Represents the presets of an `App` instance.
pub enum ApplicationType {
    App2D,
    App3D,
}

/// The `App` struct represents the common interface for many different components of the game engine.
/// It provides a unified interface for managing the application's state, input, and ECS.
pub struct App {
    title: String,
    icon: Option<Icon>,
    size: Option<LogicalSize<u32>>,
    clear_color: Option<LinearRgba>,
    input_manager: Arc<Mutex<InputManager>>,
    delta_time: f32,
    update_timer: f32,
    game_state: Option<Box<dyn Any + Send>>,
    audio: Box<dyn Audio>,
    scene: Scene,
    should_quit: bool,
    tick_systems: Vec<fn(&mut App, f32)>,
    pending_tick_add: Vec<fn(&mut App, f32)>,
    pending_tick_remove: Vec<fn(&mut App, f32)>,
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
            game_state: None,
            audio: Box::new(KiraAudio::new()),
            scene: Scene::new(),
            should_quit: false,
            tick_systems: Vec::new(),
            pending_tick_add: Vec::new(),
            pending_tick_remove: Vec::new(),
        }
    }

    /// Allows to set the title of the `App` instance.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Allows to set the icon of the `App` instance.
    pub fn with_icon(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.icon = Self::load_icon(path.as_ref());
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

    /// Allows to set a custom game state struct for the `App` instance.
    /// This allows for additional state management and control additionally to the core functionality of the engine.
    pub fn with_game_state(mut self, game_state: impl Any + Send + 'static) -> Self {
        self.game_state = Some(Box::new(game_state));
        self
    }

    /// Allows to set the preset of the `App` instance.
    /// Presets are used to quickly set up the application with a predefined configuration.
    /// Currently there are two presets available: App2D and App3D.
    /// `App2D` registers the components `Transform2D`, `Render2D`, `Camera2D`, and `Text`.
    /// `App3D` registers the components `Transform3D` and `Text`.
    /// A working out of the box 3D renderer has not been implemented yet.
    pub fn with_preset(mut self, preset: ApplicationType) -> Self {
        match preset {
            ApplicationType::App2D => {
                info!("Creating 2D app!");
                self.scene.register_component::<Transform2D>();
                self.scene.register_component::<Render2D>();
                self.scene.register_component::<Camera2D>();
                self.scene.register_component::<Text>();
            }
            ApplicationType::App3D => {
                info!("Creating 3D app!");
                self.scene.register_component::<Transform3D>();
                self.scene.register_component::<Text>();
            }
        };
        self
    }

    pub fn with_audio(mut self, audio_system: Box<dyn Audio>) -> Self {
        self.audio = audio_system;
        self
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

    /// Retrieves a reference to the registered `game_state` struct in the `App`.
    pub fn game_state<T: 'static>(&self) -> Option<&T> {
        self.game_state.as_ref()?.downcast_ref::<T>()
    }

    /// Retrieves a mutable reference to the registered `game_state` struct in the `App`.
    pub fn game_state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.game_state.as_mut()?.downcast_mut::<T>()
    }

    /// Retrieves a reference to the current `Scene` in the `App`.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Retrieves a mutable reference to the current `Scene` in the `App`
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    /// Spawns a new entity using a bundle of components.
    pub fn spawn_bundle<B: comet_ecs::Bundle>(&mut self, bundle: B) -> Entity {
        self.scene.spawn_bundle(bundle)
    }

    pub fn deferred_spawn_empty(&mut self) {
        self.scene.deferred_spawn_empty();
    }

    pub fn deferred_delete_entity(&mut self, entity: Entity) {
        self.scene.deferred_delete_entity(entity);
    }

    pub fn deferred_register_component<C: Component>(&mut self) {
        self.scene.deferred_register_component::<C>();
    }

    pub fn deferred_deregister_component<C: Component>(&mut self) {
        self.scene.deferred_deregister_component::<C>();
    }

    pub fn deferred_add_component<C: Component>(&mut self, entity: Entity, component: C) {
        self.scene.deferred_add_component::<C>(entity, component);
    }

    pub fn deferred_remove_component<C: Component>(&mut self, entity: Entity) {
        self.scene.deferred_remove_component::<C>(entity);
    }

    pub fn deferred_delete_entities_with(&mut self, components: Vec<TypeId>) {
        self.scene.deferred_delete_entities_with(components);
    }

    pub fn deferred_register_prefab(
        &mut self,
        name: impl Into<String>,
        factory: comet_ecs::PrefabFactory,
    ) {
        self.scene.deferred_register_prefab(name, factory);
    }

    pub fn deferred_spawn_prefab(&mut self, name: impl Into<String>) {
        self.scene.deferred_spawn_prefab(name);
    }

    pub fn deferred_spawn_bundle<B: comet_ecs::Bundle>(&mut self, bundle: B) {
        self.scene.deferred_spawn_bundle(bundle);
    }

    pub fn deferred_spawn_bundle_batch<B: comet_ecs::Bundle>(&mut self, bundles: Vec<B>) {
        self.scene.deferred_spawn_bundle_batch(bundles);
    }

    pub fn deferred_add_bundle<B: comet_ecs::Bundle>(&mut self, entity: Entity, bundle: B) {
        self.scene.deferred_add_bundle(entity, bundle);
    }

    pub fn apply_deferred_commands(&mut self) {
        self.scene.apply_commands();
    }

    pub fn queued_deferred_command_count(&self) -> usize {
        self.scene.queued_command_count()
    }

    pub fn query<'a, Q>(&'a mut self) -> <Q as comet_ecs::QuerySpecMut<'a>>::Builder
    where
        Q: comet_ecs::QuerySpecMut<'a>,
    {
        self.scene.query_mut::<Q>()
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

    /// Creates a new entity and returns its ID.
    pub fn new_entity(&mut self) -> Entity {
        self.scene.new_entity()
    }

    /// Deletes an entity by its ID.
    pub fn delete_entity(&mut self, entity_id: Entity) {
        self.scene.delete_entity(entity_id)
    }

    /// Gets an immutable reference to an entity by its ID.
    pub fn get_entity(&self, entity_id: Entity) -> Option<&Entity> {
        self.scene.get_entity(entity_id)
    }

    /// Registers a new component in the `Scene`.
    pub fn register_component<C: Component>(&mut self) {
        self.scene.register_component::<C>()
    }

    /// Deregisters a component from the `Scene`.
    pub fn deregister_component<C: Component>(&mut self) {
        self.scene.deregister_component::<C>()
    }

    /// Adds a component to an entity by its ID and an instance of the component.
    /// Overwrites the previous component if another component of the same type is added.
    pub fn add_component<C: Component>(&mut self, entity_id: Entity, component: C) {
        self.scene.add_component(entity_id, component)
    }

    /// Removes a component from an entity by its ID.
    pub fn remove_component<C: Component>(&mut self, entity_id: Entity) {
        self.scene.remove_component::<C>(entity_id)
    }

    /// Returns a reference to a component of an entity by its ID.
    pub fn get_component<C: Component>(&self, entity_id: Entity) -> Option<&C> {
        self.scene.get_component::<C>(entity_id)
    }

    /// Returns a mutable reference to a component of an entity by its ID.
    pub fn get_component_mut<C: Component>(&mut self, entity_id: Entity) -> Option<&mut C> {
        self.scene.get_component_mut::<C>(entity_id)
    }

    /// Deletes all entities that have the given components.
    /// The amount of queriable components is limited to 3 such that the `Archetype` creation is more efficient.
    /// Otherwise it would be a factorial complexity chaos.
    pub fn delete_entities_with(&mut self, components: Vec<TypeId>) {
        self.scene.delete_entities_with(components)
    }

    /// Returns whether an entity has the given component.
    pub fn has<C: Component>(&self, entity_id: Entity) -> bool {
        self.scene.has::<C>(entity_id)
    }

    /// Registers a prefab with the given name and factory function.
    pub fn register_prefab(&mut self, name: &str, factory: comet_ecs::PrefabFactory) {
        self.scene.register_prefab(name, factory)
    }

    /// Spawns a prefab with the given name.
    pub fn spawn_prefab(&mut self, name: &str) -> Option<Entity> {
        self.scene.spawn_prefab(name)
    }

    /// Checks if a prefab with the given name exists.
    pub fn has_prefab(&self, name: &str) -> bool {
        self.scene.has_prefab(name)
    }

    pub fn load_audio(&mut self, name: &str, path: &str) {
        self.audio.load(name, path);
    }

    pub fn play_audio(&mut self, name: &str, looped: bool) {
        self.audio.play(name, looped);
    }

    pub fn pause_audio(&mut self, name: &str) {
        self.audio.pause(name);
    }

    pub fn stop_audio(&mut self, name: &str) {
        self.audio.stop(name);
    }

    pub fn stop_all_audio(&mut self) {
        self.audio.stop_all();
    }

    pub fn update_audio(&mut self, dt: f32) {
        self.audio.update(dt);
    }

    pub fn is_playing(&self, name: &str) -> bool {
        self.audio.is_playing(name)
    }

    pub fn set_volume(&mut self, name: &str, volume: f32) {
        self.audio.set_volume(name, volume);
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

    /// Sets the amount of times the `App` game logic is updated per second
    pub fn set_update_rate(&mut self, update_rate: u32) {
        if update_rate == 0 {
            self.update_timer = f32::INFINITY;
            return;
        }
        self.update_timer = 1.0 / update_rate as f32;
    }

    fn begin_logic_tick(&mut self) {
        let baseline_tick = self.scene.change_tick().wrapping_sub(1);
        self.scene.set_query_default_tick(baseline_tick);
    }

    fn end_logic_tick(&mut self) {
        self.scene.apply_commands();
        let _ = self.scene.advance_change_tick();
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

        pollster::block_on(async {
            let update_timer = self.update_timer;
            let input_manager = self.input_manager.clone();
            let icon = self.icon.clone();
            let size = self.size.clone();
            let clear_color = self.clear_color.clone();

            let (cmd_tx, cmd_rx) = flume::unbounded::<
                <R::Handle as comet_renderer::renderer::RendererHandle>::Command,
            >();
            let (evt_tx, evt_rx) = flume::unbounded::<
                <R::Handle as comet_renderer::renderer::RendererHandle>::Event,
            >();

            let event_loop = EventLoop::new().unwrap();
            let mut renderer = R::new(
                Arc::new(Self::create_window(
                    title.clone(),
                    &icon,
                    &size,
                    &event_loop,
                )),
                clear_color,
                evt_tx,
            );
            let quit_flag = Arc::new(AtomicBool::new(false));
            let logic_quit = quit_flag.clone();
            info!("Using Renderer {}", type_name::<R>());

            info!("Setting up!");
            let logic_thread = std::thread::spawn(move || {
                let mut app = self;
                let mut handle = R::Handle::new(cmd_tx, evt_rx);
                setup(&mut app, &mut handle);

                let mut time_stack = 0.0;
                let mut last_tick = std::time::Instant::now();
                let max_steps = 5;

                while !logic_quit.load(Ordering::Relaxed) {
                    let now = std::time::Instant::now();
                    let frame_dt = now.duration_since(last_tick).as_secs_f32();
                    last_tick = now;
                    app.delta_time = frame_dt;

                    if app.dt() != f32::INFINITY {
                        time_stack += frame_dt;
                        let mut steps = 0;
                        while time_stack > app.update_timer && steps < max_steps {
                            let step = app.dt();
                            app.begin_logic_tick();
                            app.apply_tick_system_changes();
                            let mut i = 0;
                            while i < app.tick_systems.len() {
                                let system = app.tick_systems[i];
                                system(&mut app, step);
                                i += 1;
                            }
                            update(&mut app, &mut handle, step);
                            app.end_logic_tick();
                            time_stack -= app.update_timer;
                            steps += 1;
                        }
                    } else {
                        app.begin_logic_tick();
                        app.apply_tick_system_changes();
                        let mut i = 0;
                        while i < app.tick_systems.len() {
                            let system = app.tick_systems[i];
                            system(&mut app, frame_dt);
                            i += 1;
                        }
                        update(&mut app, &mut handle, frame_dt);
                        app.end_logic_tick();
                    }

                    if app.should_quit {
                        logic_quit.store(true, Ordering::Relaxed);
                        break;
                    }

                    if app.update_timer.is_finite() && app.update_timer > 0.0 {
                        let target_step = std::time::Duration::from_secs_f32(app.update_timer);
                        let elapsed = last_tick.elapsed();
                        if elapsed < target_step {
                            std::thread::sleep(target_step - elapsed);
                        }
                    } else {
                        std::thread::yield_now();
                    }
                }
            });

            let mut window_focused = true;
            let mut window_occluded = false;

            info!("Starting event loop!");
            event_loop
                .run(|event, elwt| {
                    if quit_flag.load(Ordering::Relaxed) {
                        elwt.exit()
                    }

                    if let Ok(mut manager) = input_manager.lock() {
                        manager.update(&event);
                    }

                    #[allow(unused_variables)]
                    match event {
                        Event::WindowEvent {
                            ref event,
                            window_id,
                        } => match event {
                            WindowEvent::CloseRequested {} => {
                                quit_flag.store(true, Ordering::Relaxed);
                                elwt.exit();
                            }
                            WindowEvent::Focused(focused) => {
                                window_focused = *focused;
                            }
                            WindowEvent::Occluded(occluded) => {
                                window_occluded = *occluded;
                            }
                            WindowEvent::Resized(physical_size) => {
                                renderer.resize(*physical_size);
                            }
                            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                                renderer.set_scale_factor(*scale_factor);
                            }
                            WindowEvent::RedrawRequested => {
                                while let Ok(cmd) = cmd_rx.try_recv() {
                                    renderer.apply_command(cmd);
                                }

                                if window_focused && !window_occluded {
                                    match renderer.render() {
                                        Ok(_) => {}
                                        Err(
                                            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                        ) => {
                                            let size = renderer.size();
                                            renderer.resize(size);
                                        }
                                        Err(wgpu::SurfaceError::OutOfMemory) => {
                                            error!("Out of memory!");
                                            elwt.exit();
                                        }
                                        Err(wgpu::SurfaceError::Timeout) => {
                                            warn!("Surface timeout - skipping frame");
                                        }
                                    }
                                }
                            }
                            _ => {}
                        },
                        Event::AboutToWait => {
                            while let Ok(cmd) = cmd_rx.try_recv() {
                                renderer.apply_command(cmd);
                            }

                            if window_focused && !window_occluded {
                                renderer.window().request_redraw();
                            }

                            if update_timer.is_finite() {
                                let next_frame = std::time::Instant::now()
                                    + std::time::Duration::from_secs_f32(update_timer);
                                elwt.set_control_flow(ControlFlow::WaitUntil(next_frame));
                            } else {
                                elwt.set_control_flow(ControlFlow::Wait);
                            }
                        }
                        _ => {}
                    }
                })
                .unwrap();
            logic_thread.join().ok();
        });

        info!("Shutting down {}!", title);
    }
}
