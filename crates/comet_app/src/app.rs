use std::sync::Arc;
use std::time::{Duration, Instant};
use comet_ecs::{Component, ComponentSet, Render, Renderer2D, Transform2D, World};
use comet_resources::{ResourceManager, Vertex};
use comet_renderer::{Renderer};

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
use winit::platform::windows::WindowBuilderExtWindows;
use winit_input_helper::WinitInputHelper;
use comet_input::input_handler::InputHandler;
use comet_input::keyboard::Key;

pub enum ApplicationType {
	App2D,
	App3D
}

pub struct App<'a> {
	title: &'a str,
	icon: Option<Icon>,
	size: Option<LogicalSize<u32>>,
	clear_color: Option<LinearRgba>,
	setup: Option<fn(&mut World)>,
	input_manager: WinitInputHelper,
	delta_time: f32,
	update_timer: f32,
	world: World,
	fullscreen: bool,
	should_quit: bool
}

impl<'a> App<'a> {
	pub fn new(application_type: ApplicationType) -> Self {
		let world = match application_type {
			ApplicationType::App2D => World::new("2D"),
			ApplicationType::App3D => World::new("3D"),
		};

		Self {
			title: "Untitled",
			icon: None,
			size: None,
			clear_color: None,
			setup: None,
			input_manager: WinitInputHelper::new(),
			delta_time: 0.0,
			update_timer: 0.0166667,
			world,
			fullscreen: false,
			should_quit: false
		}
	}

	pub fn with_title(mut self, title: &'a str) -> Self {
		self.title = title;
		self
	}

	pub fn with_icon(mut self, path: &'a str) -> Self {
		self.icon = Some(Self::load_icon(std::path::Path::new(path)).unwrap());
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

	pub fn with_setup(mut self, setup: fn(&mut World)) -> Self {
		self.setup = Some(setup);
		self
	}

	fn load_icon(path: &std::path::Path) -> Option<Icon> {
		let image = image::open(path).expect("Failed to open icon image");
		let rgba_image = image.to_rgba8();
		let (width, height) = rgba_image.dimensions();
		Some(Icon::from_rgba(rgba_image.into_raw(), width, height).unwrap())
	}

	pub fn world(&self) -> &World {
		&self.world
	}

	pub fn world_mut(&mut self) -> &mut World {
		&mut self.world
	}

	pub fn input_manager(&self) -> &WinitInputHelper {
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

	pub fn dt(&self) -> f32 {
		self.delta_time
	}

	pub fn quit(&mut self) {
		self.should_quit = true;
	}

	pub fn get_time_step(&self) -> f32 {
		self.update_timer
	}
	pub fn set_time_step(&mut self, time_step: f32) {
		self.update_timer = time_step;
	}

	fn create_window(app_title: &str, app_icon: &Option<Icon>, window_size: &Option<LogicalSize<u32>>, event_loop: &EventLoop<()>) -> winit::window::Window {
		let winit_window = winit::window::WindowBuilder::new()
			.with_title(app_title);

		let winit_window = if let Some(icon) = app_icon.clone() {
			winit_window.with_window_icon(Some(icon))
		}
		else {
			winit_window
		};
		let winit_window = if let Some(icon) = app_icon.clone() {
			winit_window.with_taskbar_icon(Some(icon))
		}
		else {
			winit_window
		};

		let winit_window = if let Some(size) = window_size.clone() {
			winit_window.with_inner_size(size)
		}
		else {
			winit_window
		};

		winit_window.build(event_loop).unwrap()
	}

	async fn run_app<U: Fn(&mut App, &mut Renderer, f32)>(mut self, update: U) {
		let event_loop = EventLoop::new().unwrap();
		let window = Self::create_window(self.title, &self.icon, &self.size ,&event_loop);
		let mut renderer = Renderer::new(&window, self.clear_color.clone()).await.unwrap();
		window.set_maximized(true);

		if let Some(setup) = self.setup {
			setup(&mut self.world);
		}

		renderer.initialize_atlas();

		let mut time_stack = 0.0;

		event_loop.run(|event, elwt| {
			self.delta_time = renderer.update();

			if self.should_quit {
				elwt.exit()
			}

			self.input_manager.update(&event);

			time_stack += self.delta_time;
			while time_stack > self.update_timer {
				let time = self.get_time_step();
				update(&mut self, &mut renderer, time);
				time_stack -= self.update_timer;
			}

			renderer.render_scene_2d(self.world());

			match event {
				Event::WindowEvent { ref event, window_id, }
				if window_id == renderer.window().id() => {
					match event {
						WindowEvent::CloseRequested {} => elwt.exit(),
						WindowEvent::Resized(physical_size) => {
							renderer.resize(*physical_size);
						}
						WindowEvent::RedrawRequested => {
							renderer.window().request_redraw();

							match renderer.render() {
								Ok(_) => {}
								Err(
									wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
								) => renderer.resize(renderer.size()),
								Err(wgpu::SurfaceError::OutOfMemory) => {
									error!("OutOfMemory");
									elwt.exit();
								}
								Err(wgpu::SurfaceError::Timeout) => {
									warn!("Surface timeout")
								}
							}
						}
						_ => {}
					}
				}
				_ => {}
			}
		}).unwrap();
	}

	pub fn run<U: Fn(&mut App, &mut Renderer, f32)>(mut self, update: U) {
		pollster::block_on(self.run_app(update));
	}
}