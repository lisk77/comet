use std::sync::Arc;
use comet_ecs::World;
use comet_resources::ResourceManager;
use comet_renderer::{Renderer};

use winit::{
	event::{self, *},
	event_loop::{self, EventLoop, EventLoopWindowTarget},
	keyboard::{KeyCode, PhysicalKey},
	window::{Icon, Window},
};
use comet_colors::LinearRgba;
use comet_ecs::math::Point3;
use log::warn;
use winit::dpi::{LogicalSize, PhysicalSize};

pub enum ApplicationType {
	App2D,
	App3D
}

pub struct App<'a> {
	title: &'a str,
	icon: Option<Icon>,
	size: Option<LogicalSize<u32>>,
	clear_color: Option<LinearRgba>,
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

	pub fn quit(&mut self) {
		self.should_quit = true;
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
		let winit_window = if let Some(size) = window_size.clone() {
			winit_window.with_inner_size(size)
		}
		else {
			winit_window
		};

		winit_window.build(event_loop).unwrap()
	}

	async fn run_app<F: Fn(&WindowEvent, &mut App, &mut Renderer), G: Fn(&mut World, &mut Renderer)>(mut self, input_manager: F, game_manager: G) {
		env_logger::init();
		let event_loop = EventLoop::new().unwrap();
		let window = Self::create_window(self.title, &self.icon, &self.size ,&event_loop);

		let mut renderer = Renderer::new(&window, self.clear_color.clone()).await.unwrap();
		let mut surface_configured = false;

		window.set_maximized(true);

		event_loop.run(|event, control_flow| {
			if self.should_quit {
				control_flow.exit()
			}

			game_manager(&mut self.world, &mut renderer);

			match event {
				Event::WindowEvent {
					ref event,
					window_id,
				} if window_id == renderer.window().id() => {
					match event {
						WindowEvent::CloseRequested {} => control_flow.exit(),
						WindowEvent::Resized(physical_size) => {
							surface_configured = true;
							renderer.resize(*physical_size);
						}
						WindowEvent::RedrawRequested => {
							renderer.window().request_redraw();

							if !surface_configured {
								return;
							}

							/*if self.fullscreen && !renderer.window().fullscreen().is_some() {
								renderer.resize(renderer.window().inner_size().into());
							}*/

							renderer.update();
							//println!("{}", 1.0/dt);
							match renderer.render() {
								Ok(_) => {}
								Err(
									wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
								) => renderer.resize(renderer.size()),
								Err(wgpu::SurfaceError::OutOfMemory) => {
									log::error!("OutOfMemory");
									control_flow.exit();
								}
								Err(wgpu::SurfaceError::Timeout) => {
									warn!("Surface timeout")
								}
							}
						}
						_ => { input_manager(event, &mut self, &mut renderer) }
					}
				}
				_ => {}
			}
		}).unwrap();
	}

	pub fn run<F: Fn(&WindowEvent, &mut App, &mut Renderer), G: Fn(&mut World, &mut Renderer)>(mut self, input_manager: F, game_manager: G) {
		pollster::block_on(self.run_app(input_manager, game_manager));
	}
}