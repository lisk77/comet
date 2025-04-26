use std::any::{type_name, Any};
use std::sync::{Arc, RwLock};
use comet_ecs::{Camera2D, Component, Entity, Render2D, Scene, Transform2D, Transform3D};

use winit::{
	event::*,
	event_loop::{EventLoop, EventLoopWindowTarget},
	keyboard::{KeyCode, PhysicalKey},
	window::{Icon, Window},
};
use comet_colors::{Color, LinearRgba};
use comet_log::*;
use winit::dpi::LogicalSize;
use winit_input_helper::WinitInputHelper as InputManager;
use comet_input::keyboard::Key;
use comet_renderer::renderer::Renderer;
use comet_structs::ComponentSet;

pub enum ApplicationType {
	App2D,
	App3D
}

pub struct App {
	title: String,
	icon: Option<Icon>,
	size: Option<LogicalSize<u32>>,
	clear_color: Option<LinearRgba>,
	input_manager: InputManager,
	delta_time: f32,
	update_timer: f32,
	game_state: Option<Box<dyn Any>>,
	scene: Scene,
	fullscreen: bool,
	should_quit: bool
}

impl App {
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
			scene: Scene::new(),
			fullscreen: false,
			should_quit: false
		}
	}

	pub fn with_title(mut self, title: impl Into<String>) -> Self {
		self.title = title.into();
		self
	}

	pub fn with_icon(mut self, path: impl AsRef<std::path::Path>) -> Self {
		self.icon = Self::load_icon(path.as_ref());
		self
	}

	pub fn with_size(mut self, width: u32, height: u32) -> Self {
		self.size = Some(LogicalSize::new(width, height));
		self
	}

	pub fn with_clear_color(mut self, clear_color: impl Color) -> Self {
		self.clear_color = Some(clear_color.to_linear());
		self
	}

	pub fn with_game_state(mut self, game_state: impl Any + 'static) -> Self {
		self.game_state = Some(Box::new(game_state));
		self
	}

	pub fn with_preset(mut self, preset: ApplicationType) -> Self {
		match preset {
			ApplicationType::App2D => {
				info!("Creating 2D app!");
				self.scene.register_component::<Transform2D>();
				self.scene.register_component::<Render2D>();
				self.scene.register_component::<Camera2D>()
			},
			ApplicationType::App3D => {
				info!("Creating 3D app!");
				self.scene.register_component::<Transform3D>()
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

	pub fn game_state<T: 'static>(&self) -> Option<&T> {
		self.game_state.as_ref()?.downcast_ref::<T>()
	}

	pub fn game_state_mut<T: 'static>(&mut self) -> Option<&mut T> {
		self.game_state.as_mut()?.downcast_mut::<T>()
	}

	pub fn scene(&self) -> &Scene {
		&self.scene
	}

	pub fn input_manager(&self) -> &InputManager {
		&self.input_manager
	}

	pub fn key_pressed(&self, key: Key) -> bool {
		self.input_manager.key_pressed(key)
	}

	pub fn key_held(&self, key: Key) -> bool {
		self.input_manager.key_held(key)
	}

	pub fn key_released(&self, key: Key) -> bool {
		self.input_manager.key_released(key)
	}

	pub fn new_entity(&mut self) -> usize{
		self.scene.new_entity() as usize
	}

	pub fn delete_entity(&mut self, entity_id: usize) {
		self.scene.delete_entity(entity_id)
	}

	pub fn get_entity(&self, entity_id: usize) -> Option<&Entity> {
		self.scene.get_entity(entity_id)
	}

	pub fn get_entity_mut(&mut self, entity_id: usize) -> Option<&mut Entity> {
		self.scene.get_entity_mut(entity_id)
	}

	pub fn register_component<C: Component>(&mut self) {
		self.scene.register_component::<C>()
	}

	pub fn deregister_component<C: Component>(&mut self) {
		self.scene.deregister_component::<C>()
	}

	pub fn add_component<C: Component>(&mut self, entity_id: usize, component: C) {
		self.scene.add_component(entity_id, component)
	}

	pub fn remove_component<C: Component>(&mut self, entity_id: usize) {
		self.scene.remove_component::<C>(entity_id)
	}

	pub fn get_component<C: Component>(&self, entity_id: usize) -> Option<&C> {
		self.scene.get_component::<C>(entity_id)
	}

	pub fn get_component_mut<C: Component>(&mut self, entity_id: usize) -> Option<&mut C> {
		self.scene.get_component_mut::<C>(entity_id)
	}

	pub fn get_entities_with(&self, components: ComponentSet) -> Vec<usize> {
		self.scene.get_entities_with(components)
	}

	pub fn delete_entities_with(&mut self, components: ComponentSet) {
		self.scene.delete_entities_with(components)
	}

	pub fn foreach<C: Component, K: Component>(&mut self, func: fn(&mut C,&mut K)) {
		self.scene.foreach::<C,K>(func)
	}

	pub fn has<C: Component>(&self, entity_id: usize) -> bool {
		self.scene.has::<C>(entity_id)
	}

	pub fn quit(&mut self) {
		self.should_quit = true;
	}

	pub fn dt(&self) -> f32 {
		self.update_timer
	}

	/// Sets the amount of times the game is updated per second
	pub fn set_update_rate(&mut self, update_rate: u32) {
		if update_rate == 0 {
			self.update_timer = f32::INFINITY;
			return;
		}
		self.update_timer = 1.0/update_rate as f32;
	}

	fn create_window(app_title: String, app_icon: &Option<Icon>, window_size: &Option<LogicalSize<u32>>, event_loop: &EventLoop<()>) -> Window {
		let winit_window = winit::window::WindowBuilder::new()
			.with_title(app_title);

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

	pub fn run<R: Renderer>(mut self, setup: fn(&mut App, &mut R), update: fn(&mut App, &mut R, f32)) {
		info!("Starting up {}!", self.title);

		pollster::block_on(async {
			let event_loop = EventLoop::new().unwrap();
			let window = Arc::new(Self::create_window(self.title.clone(), &self.icon, &self.size ,&event_loop));
			let mut renderer = R::new(window.clone(), self.clear_color.clone());
			info!("Renderer created! ({})", type_name::<R>());

			info!("Setting up!");
			setup(&mut self, &mut renderer);

			let mut time_stack = 0.0;

			info!("Starting event loop!");
			event_loop.run(|event, elwt| {
				self.delta_time = renderer.update();

				if self.should_quit {
					elwt.exit()
				}

				self.input_manager.update(&event);

				if self.dt() != f32::INFINITY {
					time_stack += self.delta_time;
					while time_stack > self.update_timer {
						let time = self.dt();
						update(&mut self, &mut renderer, time);
						time_stack -= self.update_timer;
					}
				}

				match event {
					Event::WindowEvent { ref event, window_id} => {
						match event {
							WindowEvent::CloseRequested {} => elwt.exit(),
							WindowEvent::Resized(physical_size) => {
								renderer.resize(*physical_size);
							}
							WindowEvent::RedrawRequested => {
								window.request_redraw();
								match renderer.render() {
									Ok(_) => {},
									Err(e) => error!("Error rendering: {}", e)
								}
							}
							_ => {}
						}
					}
					_ => {}
				}
			}).unwrap()
		});

		info!("Shutting down {}!", self.title);
	}
}