use comet::prelude::*;

fn setup(app: &mut App) {
    let clip = app.load::<AudioClip>("res://sounds/hit.ogg");
    app.play_audio(clip, true);
}

fn update(_app: &mut App, _dt: f32) {}

fn main() {
    App::with_preset(Headless)
        .with_module(AudioModule::new())
        .run(setup, update);
}
