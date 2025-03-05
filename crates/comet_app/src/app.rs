use std::any::{type_name, Any};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{Duration, Instant};
use crossbeam_channel::bounded;
use comet_ecs::{Camera2D, Component, Entity, Render, Render2D, Transform2D, Transform3D, World};
use comet_resources::{ResourceManager, Vertex};
use comet_renderer::renderer2d::Renderer2D;

use winit::{
	event::{self, *},
	event_loop::{self, EventLoop, EventLoopWindowTarget},
	keyboard::{KeyCode, PhysicalKey},
	window::{Icon, Window},
};
use comet_colors::LinearRgba;
use comet_ecs::math::Point3;
use comet_log::*;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event_loop::ControlFlow;
use winit::window::Fullscreen;
use winit_input_helper::WinitInputHelper;
use comet_input::input_handler::InputHandler;
use comet_input::keyboard::Key;
use comet_renderer::renderer::Renderer;
use crate::GameState;

pub enum ApplicationType {
	App2D,
	App3D
}

pub enum AppMessage {
	Resize(PhysicalSize<u32>),
	Input(WinitInputHelper),
	UpdateCompleted(f32),
	Quit
}

pub struct App {
	title: String,
	icon: Option<Icon>,
	size: Option<LogicalSize<u32>>,
	clear_color: Option<LinearRgba>,
	input_manager: WinitInputHelper,
	delta_time: f32,
	update_timer: f32,
	game_state: Option<Box<dyn Any>>,
	world: World,
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
			input_manager: WinitInputHelper::new(),
			delta_time: 0.0,
			update_timer: 0.0166667,
			game_state: None,
			world: World::new(),
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

	pub fn with_clear_color(mut self, clear_color: LinearRgba) -> Self {
		self.clear_color = Some(clear_color);
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
				self.world.register_component::<Transform2D>();
				self.world.register_component::<Render2D>();
				self.world.register_component::<Camera2D>()
			},
			ApplicationType::App3D => {
				info!("Creating 3D app!");
				self.world.register_component::<Transform3D>()
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

	pub fn world(&self) -> &World {
		&self.world
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

	pub fn new_entity(&mut self) -> u32 {
		self.world.new_entity()
	}

	pub fn delete_entity(&mut self, entity_id: usize) {
		self.world.delete_entity(entity_id)
	}

	pub fn get_entity(&self, entity_id: usize) -> Option<&Entity> {
		self.world.get_entity(entity_id)
	}

	pub fn get_entity_mut(&mut self, entity_id: usize) -> Option<&mut Entity> {
		self.world.get_entity_mut(entity_id)
	}

	pub fn register_component<C: Component>(&mut self) {
		self.world.register_component::<C>()
	}

	pub fn deregister_component<C: Component>(&mut self) {
		self.world.deregister_component::<C>()
	}

	pub fn add_component<C: Component>(&mut self, entity_id: usize, component: C) {
		self.world.add_component(entity_id, component)
	}

	pub fn remove_component<C: Component>(&mut self, entity_id: usize) {
		self.world.remove_component::<C>(entity_id)
	}

	pub fn get_component<C: Component>(&self, entity_id: usize) -> Option<&C> {
		self.world.get_component::<C>(entity_id)
	}

	pub fn get_component_mut<C: Component>(&mut self, entity_id: usize) -> Option<&mut C> {
		self.world.get_component_mut::<C>(entity_id)
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

	/*pub fn run<R: Renderer + 'static>(mut self, setup: fn(&mut App, &mut R), update: fn(&mut App, &mut R, f32)) {
		info!("Starting up {}!", self.title);

		pollster::block_on(async {
			let event_loop = EventLoop::new().unwrap();
			let window = Arc::new(Self::create_window(self.title.clone(), &self.icon, &self.size, &event_loop));
			let mut renderer = Arc::new(RwLock::new(R::new(window.clone(), self.clear_color.clone()).await));
			info!("Renderer created! ({})", type_name::<R>());
			window.set_maximized(true);

			let app = Arc::new(RwLock::new(self.clone()));

			// Run setup with locked app
			{
				let mut app_lock = app.write().unwrap();
				let mut renderer_lock = renderer.write().unwrap();
				setup(&mut *app_lock, &mut *renderer_lock);
			}

			let (game_tx, game_rx) = bounded::<AppMessage>(10);
			let (render_tx, render_rx) = bounded::<AppMessage>(10);

			// Spawn game logic thread
			let game_thread_app = Arc::clone(&app);
			let game_thread_renderer = Arc::clone(&renderer);

			thread::spawn(move || {
				let mut time_stack = 0.0;
				let mut last_update = Instant::now();

				while !game_thread_app.read().unwrap().should_quit {
					let now = Instant::now();
					let delta = now.duration_since(last_update).as_secs_f32();

					// Get a single write lock and use it for the entire update
					let mut app_lock = game_thread_app.write().unwrap();
					app_lock.delta_time = delta;

					time_stack += delta;
					let update_timer = app_lock.update_timer; // Store the timer value

					while time_stack > update_timer {
						let mut renderer_lock = game_thread_renderer.write().unwrap();
						update(&mut *app_lock, &mut *renderer_lock, delta);
						drop(renderer_lock);
						time_stack -= update_timer;
						render_tx.send(AppMessage::UpdateCompleted(delta)).unwrap();
					}

					// Lock is automatically released here
					drop(app_lock);

					last_update = now;
					thread::sleep(Duration::from_millis(1));
				}
			});

			// Main thread handles events and rendering
			info!("Starting event loop!");
			event_loop.run(move |event, elwt| {
				// Get a single write lock for the entire input handling
				let mut app_lock = app.write().unwrap();
				if app_lock.should_quit {
					elwt.exit();
				}
				app_lock.input_manager.update(&event);
				drop(app_lock); // Explicitly drop the lock before event handling

				match event {
					Event::WindowEvent { ref event, .. } => {
						match event {
							WindowEvent::CloseRequested => {
								app.write().unwrap().quit();
								elwt.exit();
							}
							WindowEvent::Resized(size) => {
								let mut renderer_lock = renderer.write().unwrap();
								renderer_lock.resize(*size);
							}
							WindowEvent::RedrawRequested => {
								while let Ok(AppMessage::UpdateCompleted(_)) = render_rx.try_recv() {
									let mut renderer_lock = renderer.write().unwrap();
									match renderer_lock.render() {
										Ok(_) => window.request_redraw(),
										Err(e) => error!("Error rendering: {}", e)
									}
								}
							}
							_ => {}
						}
					}
					_ => {}
				}
			}).unwrap();
		});

		info!("Shutting down {}!", self.title.clone());
	}*/

	pub fn run<R: Renderer>(mut self, setup: fn(&mut App, &mut R), update: fn(&mut App, &mut R, f32)) {
		info!("Starting up {}!", self.title);

		pollster::block_on(async {
			let event_loop = EventLoop::new().unwrap();
			let window = Arc::new(Self::create_window(self.title.clone(), &self.icon, &self.size ,&event_loop));
			let mut renderer = R::new(window.clone(), self.clear_color.clone()).await;
			info!("Renderer created! ({})", type_name::<R>());
			//window.set_maximized(true);

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