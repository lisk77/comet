use comet::prelude::*;
use winit_input_helper::WinitInputHelper;
use comet_input::keyboard::Key;

fn setup(app: &mut App, renderer: &mut Renderer2D) {
	// Takes all the textures from resources/textures and puts them into a texture atlas
	renderer.initialize_atlas();

	let camera = app.new_entity();
	app.add_component(camera, Transform2D::new());
	app.add_component(camera, Camera2D::new(Vec2::new(2.0, 2.0), 1.0, 1));

	let e1 = app.new_entity();

	app.add_component(e1, Transform2D::new());

	let mut renderer2d = Render2D::new();
	renderer2d.set_texture(r"resources/textures/comet_icon.png");
	renderer2d.set_visibility(true);

	app.add_component(e1, renderer2d);
}

fn update(app: &mut App, renderer: &mut Renderer2D, dt: f32) {
	handle_input(app, dt);

	renderer.render_scene_2d(app.scene());
}

fn handle_input(app: &mut App, dt: f32) {
	if app.key_held(Key::KeyW)
		|| app.key_held(Key::KeyA)
		|| app.key_held(Key::KeyS)
		|| app.key_held(Key::KeyD)
	{
		update_position(
			app.input_manager().clone(),
			app.get_component_mut::<Transform2D>(1).unwrap(),
			dt
		);
	}
}

fn update_position(input: WinitInputHelper, transform: &mut Transform2D, dt: f32) {
	let mut direction = Vec2::ZERO;

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

	// If check to prevent division by zero and the comet to fly off into infinity...
	if direction != Vec2::ZERO {
		let normalized_dir = direction.normalize();
		let displacement = normalized_dir * 777.7 * dt;
		transform.translate(displacement);
	}
}

fn main() {
	App::new()
		.with_title("Simple Move 2D")
		.with_preset(App2D)
		.run::<Renderer2D>(setup, update);
}