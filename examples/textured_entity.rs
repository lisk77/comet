use comet::prelude::*;

fn setup(app: &mut App, renderer: &mut Renderer2D) {
	// Creating a texture atlas from the provided textures in the vector
	renderer.set_texture_atlas(vec!["./resources/textures/comet_icon.png".to_string()]);

	// Creating a camera entity
	let cam = app.new_entity();
	app.add_component(cam, Transform2D::new());
	app.add_component(cam, Camera2D::new(Vec2::new(2.0, 2.0), 1.0, 1));

	// Creating a textured entity
	let e0 = app.new_entity();
	app.add_component(e0, Transform2D::new());

	let mut render = Render2D::new();
	render.set_visibility(true);
	render.set_texture("./resources/textures/comet_icon.png");

	app.add_component(e0, render);
}

fn update(app: &mut App, renderer: &mut Renderer2D, dt: f32) {
	renderer.render_scene_2d(app.scene())
}

fn main() {
	App::new()
		.with_title("Textured Entity")
		.with_preset(App2D)
		.run::<Renderer2D>(setup, update);
}