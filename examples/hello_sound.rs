use comet::prelude::*;

fn setup(app: &mut App, _renderer: &mut Renderer2D) {
    app.load_audio("startup", "res/sounds/hit.ogg");
    app.play_audio("startup", true); // second parameter loops the sound

    // here the float indicated the volume percentage
    // in the standard backend for audio in comet (kira) 0.0 is equal to -20dB and 1.0 to 0dB
    app.set_volume("startup", 1.0);
}

#[allow(unused_variables)]
fn update(app: &mut App, renderer: &mut Renderer2D, dt: f32) {
    // in this example, update_audio doesnt do anything because the kira audio system
    // doesnt need any update
    app.update_audio(dt);
}

fn main() {
    App::new()
        .with_title("Comet Audio Example")
        .run::<Renderer2D>(setup, update);
}
