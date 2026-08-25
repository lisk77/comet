use comet::prelude::*;

// Spawning this tag also inserts its required components.
#[derive(Component)]
#[require(Transform, Sprite = player_sprite)]
struct Player;

fn player_sprite() -> Sprite {
    Sprite::with_texture("res://textures/comet-128.png")
}

fn setup(app: &mut App) {
    app.spawn(Camera2d);
    // You can also add your own specific components even if they are already required
    // app.spawn((Player, Transform::at(v3::ZERO)));
    // This will still add the Sprite component but overrides the Transform component
    // with your self defined variant
    app.spawn(Player);
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
        .with_title("Required Components")
        .run(setup, update);
}
