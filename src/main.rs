use std::ops::Deref;
use comet::{
	app::{
		App,
		ApplicationType::*
	},
	renderer::renderer2d::Renderer2D,
	ecs::{
		Render2D,
		Transform2D,
		Component,
		Render
	},
	math::*,
	input::keyboard::*,
	log::*
};

use winit_input_helper::WinitInputHelper;
use comet_input::input_handler::InputHandler;

#[derive(Debug, Clone)]
struct GameState {
	running: bool
}

impl GameState {
	pub fn new() -> Self {
		Self {
			running: true
		}
	}

	pub fn is_running(&self) -> bool {
		self.running
	}

	pub fn set_running(&mut self, running: bool) {
		self.running = running;
	}
}

fn update_position(input: WinitInputHelper, transform: &mut Transform2D, dt: f32) {
	let mut direction = Vec2::ZERO;
	let previous = transform.position().clone();

	if input.key_held(Key::KeyW) {
		direction += Vec2::Y;
	}
	if input.key_held(Key::KeyA) {
		direction -= Vec2::X;
	}
	if input.key_held(Key::KeyS) {
		direction -= Vec2::Y;
	}
	if input.key_held(Key::KeyD) {
		direction += Vec2::X;
	}

	if direction != Vec2::ZERO {
		let normalized_dir = direction.normalize();
		if normalized_dir.x().is_nan() || normalized_dir.y().is_nan() {
			error!("Direction is NaN! X: {}, Y: {}", normalized_dir.x(), normalized_dir.y());
		}
		let displacement = normalized_dir * 777.7 * dt;
		transform.translate(displacement);
	}
}

fn handle_input(app: &mut App, dt: f32) {
	if app.key_pressed(Key::NumpadAdd) { debug!("pressed +"); app.set_update_rate(120); }
	if app.key_pressed(Key::Minus) { app.set_update_rate(60); }
	if app.key_pressed(Key::KeyQ) { app.quit() }
	if app.key_pressed(Key::KeyE) { app.world_mut().get_component_mut::<Transform2D>(0).translate([0f32,0f32].into()) }
	if app.key_held(Key::KeyW)
		|| app.key_held(Key::KeyA)
		|| app.key_held(Key::KeyS)
		|| app.key_held(Key::KeyD)
	{
		update_position(app.input_manager().clone(), app.world_mut().get_component_mut::<Transform2D>(0), dt);
	}
}

fn setup(app: &mut App, renderer: &mut Renderer2D) {
	renderer.initialize_atlas();
	renderer.load_shader(None, "blacknwhite.wgsl");
	renderer.load_shader(None, "crt.wgsl");
	renderer.load_shader(None, "glitch.wgsl");
	renderer.apply_shader("glitch.wgsl");

	let world = app.world_mut();
	world.register_component::<Render2D>();
	//world.register_component::<Rectangle2D>();

	let mut renderer2d = Render2D::new();
	renderer2d.set_texture(r"resources/textures/comet_icon.png");
	renderer2d.set_visibility(true);

	let id = world.new_entity();
	world.add_component(id as usize, renderer2d.clone());

	let transform = world.get_component_mut::<Transform2D>(id as usize);
	transform.translate(Vec2::X*5.0);

//	world.add_component(id as usize, renderer2d);

	/*let rectangle2d = Rectangle2D::new(*tranform.position(), Vec2::new(0.1, 0.1));
	world.add_component(id as usize, rectangle2d);

	let id2 = world.new_entity();
	let tranform2 = world.get_component_mut::<Transform2D>(id as usize);
	let rectangle = Rectangle2D::new(*tranform2.position(), Vec2::new(0.1, 0.1));

	world.add_component(id2 as usize, rectangle);*/

}

fn update(app: &mut App, renderer: &mut Renderer2D, dt: f32) {
	let is_running = app.game_state::<GameState>().map(|gs| gs.is_running()).unwrap_or(false);

	match is_running {
		true => handle_input(app, dt),
		false => {}
	}

	if app.key_pressed(Key::KeyP) { app.game_state_mut::<GameState>().map(|gs| gs.set_running(!gs.is_running())); }
	if app.key_pressed(Key::KeyC) { renderer.apply_shader("blacknwhite.wgsl"); }
	if app.key_pressed(Key::KeyR) { renderer.apply_base_shader(); }

	renderer.render_scene_2d(app.world());
}

fn main() {
	App::new(App2D)
		.with_title("Comet App")
		.with_icon(r"resources/textures/comet_icon.png")
		.with_size(1920, 1080)
		.with_game_state(GameState::new())
		.run::<Renderer2D>(setup, update)
	;
}