use comet::prelude::*;

// Zero sized components are also called tags in the documentation
#[derive(Component)]
struct Player;

fn setup(app: &mut App, _renderer: &mut RenderHandle2D) {
    app.register_component::<Player>();

    app.spawn((Transform2D::new(), Camera2D::new(v2::new(2.0, 2.0), 1.0, 1)));

    app.spawn((
        Player,
        Transform2D::new(),
        Render2D::with_texture("res://textures/comet_icon.png"),
    ));
}

fn update(app: &mut App, renderer: &mut RenderHandle2D, dt: f32) {
    handle_input(app, dt);

    renderer.render_scene_2d(app.scene_mut());
}

fn handle_input(app: &mut App, dt: f32) {
    let mut direction = v2::ZERO;
    if app.key_held(Key::KeyW) {
        direction += v2::Y;
    }
    if app.key_held(Key::KeyA) {
        direction -= v2::X;
    }
    if app.key_held(Key::KeyS) {
        direction -= v2::Y;
    }
    if app.key_held(Key::KeyD) {
        direction += v2::X;
    }

    if direction != v2::ZERO {
        app.query::<&mut Transform2D, With<Player>>().for_each(|t| {
            t.translate(direction.normalize() * 777.7 * dt);
        });
    }
}

fn main() {
    App::new()
        .with_title("Simple Move 2D")
        .with_preset(App2D)
        .run::<Renderer2D>(setup, update);
}
