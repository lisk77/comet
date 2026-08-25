use comet::prelude::*;

fn setup(app: &mut App) {
    let clip = app.load::<AudioClip>("res://sounds/hit.ogg");
    app.spawn((
        AudioSource::new(clip),
        PlaybackSettings::LOOP.with_volume(0.5),
    ));
}

fn update(app: &mut App, _dt: f32) {
    if !app.key_pressed(Key::Space) {
        return;
    }

    app.query::<&mut PlaybackState, ()>()
        .for_each(|state| match state {
            PlaybackState::Playing => state.pause(),
            PlaybackState::Paused => state.play(),
            PlaybackState::Stopped | PlaybackState::Finished => {}
        });
}

fn main() {
    App::with_preset(App2D)
        .with_title("Simple Audio")
        .run(setup, update);
}
