use comet::prelude::*;

#[derive(Component)]
struct Player;

// Bundles name reusable groups of components.
bundle!(Comet {
    player: Player,
    transform: Transform,
    sprite: Sprite
});

fn setup(app: &mut App) {
    app.spawn(Camera2d);
    app.spawn(Comet {
        player: Player,
        transform: Transform::new(),
        sprite: Sprite::with_texture("res://textures/comet-128.png"),
    });
}

fn update(app: &mut App, dt: f32) {
    let direction = movement_direction(app);

    move_players(app.query::<&mut Transform, With<Player>>(), direction, dt);
}

fn movement_direction(app: &App) -> v2 {
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
    direction
}

fn move_players(players: Query<&mut Transform, With<Player>>, direction: v2, dt: f32) {
    if direction == v2::ZERO {
        return;
    }

    let displacement = direction.normalize() * 777.7 * dt;
    for transform in players {
        transform.translate(displacement.into());
    }
}

fn main() {
    App::with_preset(App2D)
        .with_title("Bundles")
        .run(setup, update);
}
