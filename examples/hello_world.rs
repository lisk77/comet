use comet::prelude::*;

fn setup(_app: &mut App) {}

fn update(_app: &mut App, _dt: f32) {}

fn main() {
    App::with_preset(App2D)
        .with_title("Hello World")
        .run(setup, update);
}
