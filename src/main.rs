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
use comet_ecs::{Component, ComponentSet, Transform2D};
use comet_input::mouse::{mouse_entered, mouse_pressed, Button};

#[derive(Component)]
struct TestComponent {
	position: f32
}

fn input(event: &WindowEvent, app: &mut App, renderer: &mut Renderer) {
	match event {
		_ if key_pressed(event, Key::KeyI) => app.world_mut().register_component::<TestComponent>(),
		_ if key_pressed(event, Key::Escape) => app.quit(),
		_ if key_pressed(event, Key::KeyE) => {
			let id = app.world_mut().new_entity();
			app.world_mut().add_component::<TestComponent>(id as usize, TestComponent::new())
		},
		_ if key_pressed(event, Key::KeyR) => {
			debug!(format!("{:?}", app.world().get_entities_with(ComponentSet::from_ids(vec![Transform2D::type_id(), TestComponent::type_id()]))));
		},
		_ => {}
	}
}

fn update(world: &mut World, renderer: &mut Renderer) {
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