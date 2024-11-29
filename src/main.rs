use comet::{
	app::{
		App,
		ApplicationType::*
	},
	renderer::Renderer2D,
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

	if (transform.position().as_vec() - previous.as_vec()).x() > 13.0 {
		debug!("Actual Displacement: {:?}", transform.position().as_vec() - previous.as_vec());
	}
}

fn setup(app: &mut App, renderer: &mut Renderer2D) {
	renderer.initialize_atlas();

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

	world.add_component(id as usize, renderer2d);

	/*let rectangle2d = Rectangle2D::new(*tranform.position(), Vec2::new(0.1, 0.1));
	world.add_component(id as usize, rectangle2d);

	let id2 = world.new_entity();
	let tranform2 = world.get_component_mut::<Transform2D>(id as usize);
	let rectangle = Rectangle2D::new(*tranform2.position(), Vec2::new(0.1, 0.1));

	world.add_component(id2 as usize, rectangle);*/

}

fn update(app: &mut App, renderer: &mut Renderer2D, dt: f32) {
	if app.key_pressed(Key::Escape) { app.quit() }
	if app.key_pressed(Key::KeyP) {
		if app.dt() == f32::INFINITY { app.set_update_rate(60) }
		else {
			app.set_update_rate(0);
		}
	}
	if app.key_pressed(Key::KeyE) { app.world_mut().get_component_mut::<Transform2D>(0).translate([0f32,0f32].into()) }
	if app.key_held(Key::KeyW)
		|| app.key_held(Key::KeyA)
		|| app.key_held(Key::KeyS)
		|| app.key_held(Key::KeyD)
	{
		update_position(app.input_manager().clone(), app.world_mut().get_component_mut::<Transform2D>(0), dt);
	}

	let mut transform = app.world_mut().get_component_mut::<Transform2D>(0);

	renderer.render_scene_2d(app.world());
}

fn main() {
	App::new(App2D)
		.with_title("Comet App")
		.with_icon(r"resources/textures/comet_icon.png")
		.with_size(1920, 1080)
		.run::<Renderer2D>(setup, update)
	;
}