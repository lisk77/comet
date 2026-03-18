// This is the simple_move_2d example but using prefabs
use comet::prelude::*;

#[derive(Component)]
struct Player;

fn setup(app: &mut App, renderer: &mut RenderHandle2D) {
    renderer.init_atlas();

    app.register_component::<Player>();

    register_prefab!(
        app,
        "camera",
        Transform2D::new(),
        Camera2D::new(v2::new(2.0, 2.0), 1.0, 1)
    );

    register_prefab!(
        app,
        "player",
        Player,
        Transform2D::new(),
        Render2D::with_texture("res://textures/comet_icon.png")
    );

    app.spawn_prefab("camera");
    app.spawn_prefab("player");
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
        .with_preset(App2D)
        .with_title("Prefabs Example")
        .run::<Renderer2D>(setup, update);
}
