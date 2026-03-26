use comet::prelude::*;

fn setup(app: &mut App, _renderer: &mut RenderHandle2D) {
    app.load::<AudioClip>("res://sounds/hit.ogg");
    // The name of the audio is the file name without the extension
    app.play_audio("hit", true);
}

fn update(app: &mut App, _renderer: &mut RenderHandle2D, dt: f32) {
    // In this case, update_audio is a no op though other engines could update
    // through this method
    app.update_audio(dt);
}

fn main() {
    App::new()
        .with_title("Comet Sound Example")
        // This can be used to add your own sound engine to the engine
        .with_audio(Box::new(comet_audio::KiraAudio::new()))
        .run::<Renderer2D>(setup, update);
}
