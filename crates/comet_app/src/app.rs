use comet_colors::{Color as ColorTrait, LinearRgba};
use comet_ecs::{Camera2D, Component, Entity, Render2D, Scene, Text, Transform2D, Transform3D};
use comet_input::keyboard::Key;
use comet_log::*;
use comet_renderer::renderer::Renderer;
use comet_sound::*;
use std::any::{type_name, Any, TypeId};
use std::sync::Arc;
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
    input_manager: InputManager,
    delta_time: f32,
    update_timer: f32,
    game_state: Option<Box<dyn Any>>,
    audio: Box<dyn Audio>,
    scene: Scene,
    should_quit: bool,
}

impl App {
    /// Creates a new `App` instance.
    pub fn new() -> Self {
        Self {
            title: "Untitled".to_string(),
            icon: None,
            size: None,
            clear_color: None,
            input_manager: InputManager::new(),
            delta_time: 0.0,
            update_timer: 0.0166667,
            game_state: None,
            audio: Box::new(KiraAudio::new()),
            scene: Scene::new(),
            should_quit: false,
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
    pub fn with_game_state(mut self, game_state: impl Any + 'static) -> Self {
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

    /// Retrieves a reference to the `InputManager`.
    pub fn input_manager(&self) -> &InputManager {
        &self.input_manager
    }

    /// Checks if a key is currently pressed.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.input_manager.key_pressed(key)
    }

    /// Checks if a key is currently held.
    pub fn key_held(&self, key: Key) -> bool {
        self.input_manager.key_held(key)
    }

    /// Checks if a key was released this frame.
    pub fn key_released(&self, key: Key) -> bool {
        self.input_manager.key_released(key)
    }

    /// Creates a new entity and returns its ID.
    pub fn new_entity(&mut self) -> usize {
        self.scene.new_entity() as usize
    }

    /// Deletes an entity by its ID.
    pub fn delete_entity(&mut self, entity_id: usize) {
        self.scene.delete_entity(entity_id)
    }

    /// Gets an immutable reference to an entity by its ID.
    pub fn get_entity(&self, entity_id: usize) -> Option<&Entity> {
        self.scene.get_entity(entity_id)
    }

    /// Gets a mutable reference to an entity by its ID.
    pub fn get_entity_mut(&mut self, entity_id: usize) -> Option<&mut Entity> {
        self.scene.get_entity_mut(entity_id)
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
    pub fn add_component<C: Component>(&mut self, entity_id: usize, component: C) {
        self.scene.add_component(entity_id, component)
    }

    /// Removes a component from an entity by its ID.
    pub fn remove_component<C: Component>(&mut self, entity_id: usize) {
        self.scene.remove_component::<C>(entity_id)
    }

    /// Returns a reference to a component of an entity by its ID.
    pub fn get_component<C: Component>(&self, entity_id: usize) -> Option<&C> {
        self.scene.get_component::<C>(entity_id)
    }

    /// Returns a mutable reference to a component of an entity by its ID.
    pub fn get_component_mut<C: Component>(&mut self, entity_id: usize) -> Option<&mut C> {
        self.scene.get_component_mut::<C>(entity_id)
    }

    /// Returns a list of entities that have the given components.
    /// The amount of queriable components is limited to 3 such that the `Archetype` creation is more efficient.
    /// Otherwise it would be a factorial complexity chaos.
    pub fn get_entities_with(&self, components: Vec<TypeId>) -> Vec<usize> {
        self.scene.get_entities_with(components)
    }

    /// Deletes all entities that have the given components.
    /// The amount of queriable components is limited to 3 such that the `Archetype` creation is more efficient.
    /// Otherwise it would be a factorial complexity chaos.
    pub fn delete_entities_with(&mut self, components: Vec<TypeId>) {
        self.scene.delete_entities_with(components)
    }

    /// Iterates over all entities that have the two given components and calls the given function.
    pub fn foreach<C: Component, K: Component>(&mut self, func: fn(&mut C, &mut K)) {
        self.scene.foreach::<C, K>(func)
    }

    /// Returns whether an entity has the given component.
    pub fn has<C: Component>(&self, entity_id: usize) -> bool {
        self.scene.has::<C>(entity_id)
    }

    /// Registers a prefab with the given name and factory function.
    pub fn register_prefab(&mut self, name: &str, factory: comet_ecs::PrefabFactory) {
        self.scene.register_prefab(name, factory)
    }

    /// Spawns a prefab with the given name.
    pub fn spawn_prefab(&mut self, name: &str) -> Option<usize> {
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

    /// Sets the amount of times the `App` game logic is updated per second
    pub fn set_update_rate(&mut self, update_rate: u32) {
        if update_rate == 0 {
            self.update_timer = f32::INFINITY;
            return;
        }
        self.update_timer = 1.0 / update_rate as f32;
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
        mut self,
        setup: fn(&mut App, &mut R),
        update: fn(&mut App, &mut R, f32),
    ) {
        info!("Starting up {}!", self.title);

        pollster::block_on(async {
            let event_loop = EventLoop::new().unwrap();
            let window = Arc::new(Self::create_window(
                self.title.clone(),
                &self.icon,
                &self.size,
                &event_loop,
            ));
            let mut renderer = R::new(window.clone(), self.clear_color.clone());
            info!("Renderer created! ({})", type_name::<R>());

            info!("Setting up!");
            setup(&mut self, &mut renderer);

            let mut time_stack = 0.0;
            let mut window_focused = true;
            let mut window_occluded = false;

            info!("Starting event loop!");
            event_loop
                .run(|event, elwt| {
                    if self.should_quit {
                        elwt.exit()
                    }

                    self.input_manager.update(&event);

                    #[allow(unused_variables)]
                    match event {
                        Event::WindowEvent {
                            ref event,
                            window_id,
                        } => match event {
                            WindowEvent::CloseRequested {} => elwt.exit(),
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
                                if window_focused && !window_occluded {
                                    match renderer.render() {
                                        Ok(_) => {}
                                        Err(
                                            wgpu::SurfaceError::Lost
                                            | wgpu::SurfaceError::Outdated,
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
                            self.delta_time = renderer.update();

                            if self.dt() != f32::INFINITY {
                                time_stack += self.delta_time;
                                while time_stack > self.update_timer {
                                    let time = self.dt();
                                    update(&mut self, &mut renderer, time);
                                    time_stack -= self.update_timer;
                                }
                            }

                            if window_focused && !window_occluded {
                                window.request_redraw();
                            }

                            if self.dt().is_finite() {
                                let next_frame = std::time::Instant::now()
                                    + std::time::Duration::from_secs_f32(self.update_timer);
                                elwt.set_control_flow(ControlFlow::WaitUntil(next_frame));
                            } else {
                                elwt.set_control_flow(ControlFlow::Wait);
                            }
                        }
                        _ => {}
                    }
                })
                .unwrap()
        });

        info!("Shutting down {}!", self.title);
    }
}
