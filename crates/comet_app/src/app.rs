use std::sync::Arc;
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

	pub fn render_scene_2d(&self, renderer: &mut Renderer) {
		let entities =  self.world.get_entities_with(ComponentSet::from_ids(vec![Renderer2D::type_id()]));
		let mut vertex_buffer: Vec<Vertex> = Vec::new();
		let mut index_buffer: Vec<u16> = Vec::new();

		for entity in entities {
			let renderer_component =  self.world().get_component::<Renderer2D>(entity as usize);
			let transform_component = self.world().get_component::<Transform2D>(entity as usize);

			if renderer_component.is_visible() {
				//renderer.draw_texture_at(renderer_component.get_texture(), Point3::new(transform_component.position().x(), transform_component.position().y(), 0.0));
				let position = transform_component.position();
				let region = renderer.get_texture(renderer_component.get_texture().to_string());
				let (dim_x, dim_y) = region.dimensions();

				let (bound_x, bound_y) =
					((dim_x as f32/ renderer.config().width as f32) * 0.5, (dim_y as f32/ renderer.config().height as f32) * 0.5);

				let buffer_size = vertex_buffer.len() as u16;

				vertex_buffer.append(&mut vec![
					Vertex :: new ( [-bound_x + position.x(),  bound_y + position.y(), 0.0], [region.x0(), region.y0()] ),
					Vertex :: new ( [-bound_x + position.x(), -bound_y + position.y(), 0.0], [region.x0(), region.y1()] ),
					Vertex :: new ( [ bound_x + position.x(), -bound_y + position.y(), 0.0], [region.x1(), region.y1()] ) ,
					Vertex :: new ( [ bound_x + position.x(),  bound_y + position.y(), 0.0], [region.x1(), region.y0()] )
				]);

				index_buffer.append(&mut vec![
					0 + buffer_size, 1 + buffer_size, 3 + buffer_size,
					1 + buffer_size, 2 + buffer_size, 3 + buffer_size
				]);
			}
		}

		renderer.set_buffers(vertex_buffer, index_buffer);

		/*for entity in entities {
			let renderer_component = self.world.get_component::<Renderer2D>(entity as usize);
			let position_component = self.world.get_component::<Transform2D>(entity as usize);
			if renderer_component.is_visible() {
				renderer.draw_texture_at(renderer_component.get_texture(), Point3::new(position_component.position().x(), position_component.position().y(), 0.0));
			}
		}*/
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

		renderer.initialize_atlas();

		event_loop.run(|event, control_flow| {
			if self.should_quit {
				control_flow.exit()
			}
			game_manager(&mut self.world, &mut renderer);
			self.render_scene_2d(&mut renderer);

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
						_ => {
							input_manager(event, &mut self, &mut renderer);
						}
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