// This is the simple_move_2d example but using bundles
use comet::prelude::*;

#[derive(Component)]
struct Player;

// Bundles are a convenient way to name groups of components that are getting
// reused in code multiple times.
bundle!(Comet {
    player: Player,
    transform: Transform,
    render: Sprite
});

fn setup(app: &mut App) {
    app.register_component::<Player>();

    app.spawn(Camera2d::new(1.0, 1));

    // notice that bundles are being spawned by `spawn` as well
    app.spawn(Comet {
        player: Player,
        transform: Transform::new(),
        render: Sprite::with_texture("res://textures/comet-128.png"),
    });
}

fn update(app: &mut App, dt: f32) {
    handle_input(app, dt);
}

fn handle_input(app: &mut App, dt: f32) {
    let mut direction = v2::ZERO;
    if app.key_held(Key::KeyW) { direction += v2::Y; }
    if app.key_held(Key::KeyA) { direction -= v2::X; }
    if app.key_held(Key::KeyS) { direction -= v2::Y; }
    if app.key_held(Key::KeyD) { direction += v2::X; }

    if direction != v2::ZERO {
        app.query::<&mut Transform, With<Player>>().for_each(|t| {
            let normalized_dir = direction.normalize();
            let displacement = normalized_dir * 777.7 * dt;
            t.translate(displacement.into());
        });
    }
}

fn main() {
    App::with_preset(App2D)
        .with_title("Bundles Example")
        .run(setup, update);
}
