use comet::prelude::*;

fn setup(app: &mut App) {
    // Load the audio clip (name for playback is the file stem: "hit")
    app.load::<AudioClip>("res://sounds/hit.ogg");
    app.play_audio("hit", true);
}

fn update(_app: &mut App, _dt: f32) {}

fn main() {
    App::with_preset(Headless)
        .with_module(AudioModule::new())
        .run_headless(setup, update);
}
