use comet::{
	app::{
		App,
		ApplicationType::*
	},
	renderer::Renderer,
	ecs::World,
	math::*,
	input::keyboard::*,
	log::*
};
use winit::event::{WindowEvent};
use comet_ecs::{Component, ComponentSet, Render, Renderer2D, Transform2D};
use comet_input::mouse::{mouse_entered, mouse_pressed, Button};

fn input(event: &WindowEvent, app: &mut App, renderer: &mut Renderer) {
	match event {
		_ if key_pressed(event, Key::Escape) => app.quit(),
		_ if key_pressed(event, Key::KeyC) => { renderer.clear_buffers() }
		_ if key_pressed(event, Key::KeyE) => {
			let mut renderer2d = Renderer2D::new();
			renderer2d.set_texture(r"resources/textures/comet_icon.png");
			renderer2d.set_visibility(true);

			let id = app.world_mut().new_entity();
			app.world_mut().add_component(id as usize, renderer2d.clone());
			app.world_mut().add_component(0, renderer2d);

			let transform = app.world_mut().get_component_mut::<Transform2D>(id as usize);
			transform.position_mut().set_x(0.5);

			debug!(format!("{:?}", app.world().components().get_component::<Renderer2D>(0)));
		},
		_ if key_pressed(event, Key::KeyW) => {
			let transform = app.world_mut().get_component_mut::<Transform2D>(0);
			let y = transform.position().y();
			transform.position_mut().set_y(y + 0.1);
		},
		_ if key_pressed(event, Key::KeyA) => {
			let transform = app.world_mut().get_component_mut::<Transform2D>(0);
			let x = transform.position().x();
			transform.position_mut().set_x(x - 0.1);
		},
		_ if key_pressed(event, Key::KeyS) => {
			let transform = app.world_mut().get_component_mut::<Transform2D>(0);
			let y = transform.position().y();
			transform.position_mut().set_y(y - 0.1);
		},
		_ if key_pressed(event, Key::KeyD) => {
			let transform = app.world_mut().get_component_mut::<Transform2D>(0);
			let x = transform.position().x();
			transform.position_mut().set_x(x + 0.1);
		}
		_ => {}
	}
}

fn update(world: &mut World, renderer: &mut Renderer) {
	if !world.components().contains_components(ComponentSet::from_ids(vec![Transform2D::type_id(), Renderer2D::type_id()])) {
		world.register_component::<Renderer2D>();
	}
	if world.entities().len() == 0 {
		let id = world.new_entity();
	}
}

fn main() {
	App::new(App2D)
		.with_title("Comet App")
		.with_icon(r"resources/textures/comet_icon.png")
		.with_size(1920, 1080)
		.run(input, update)
	;
}