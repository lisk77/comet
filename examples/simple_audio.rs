use comet::prelude::*;

fn setup(app: &mut App, _renderer: &mut RenderHandle2D) {
    // Load the audio clip (name for playback is the file stem: "hit")
    app.load::<AudioClip>("res://sounds/hit.ogg");
    app.play_audio("hit", true);
}

fn update(_app: &mut App, _renderer: &mut RenderHandle2D, _dt: f32) {}

fn main() {
    App::new()
        .with_title("Comet Sound Example")
        .with_module(AudioModule::new())
        .run::<Renderer2D>(setup, update);
}
