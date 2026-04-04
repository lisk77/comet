use comet::prelude::*;

fn setup(app: &mut App) {
    // Creating a camera entity
    app.spawn((Transform::new(), Camera2d::new(v2::new(2.0, 2.0), 1.0, 1)));

    // Creating a textured entity
    app.spawn((
        Transform::new(),
        Sprite::with_texture("res://textures/comet_icon.png"),
    ));
}

fn update(_app: &mut App, _dt: f32) {}

fn main() {
    App::with_preset(App2D)
        .with_title("Textured Entity")
        .run(setup, update);
}
