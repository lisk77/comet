use comet::prelude::*;

fn setup(app: &mut App) {
    // Creating a camera entity
    app.spawn_bundle(Camera2d::new(1.0, 1));

    // Creating a textured entity
    app.spawn((
        Transform::new(),
        Sprite::with_texture("res://textures/comet-128.png"),
    ));
}

fn update(_app: &mut App, _dt: f32) {}

fn main() {
    App::with_preset(App2D)
        .with_title("Textured Entity")
        .run(setup, update);
}
