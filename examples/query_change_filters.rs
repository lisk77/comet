use comet::prelude::*;

#[derive(Component, Default)]
struct Count(u8);

fn setup(app: &mut App) {
    app.spawn(Count::new());
}

fn update(app: &mut App, _dt: f32) {
    // Setup and the first update are handled as the same tick.
    if app.query::<&Count, Added<Count>>().count() != 0 {
        info!("Count was added this tick");
    }

    increment_count(app);

    if app.query::<&Count, Changed<Count>>().count() != 0 {
        info!(
            "Count was changed to {}",
            app.query::<&Count, ()>().next().unwrap().0
        );
    }

    if let Some((entity, count)) = app.query::<(Entity, &Count), ()>().next() {
        if count.0 == 10 {
            info!("Count reached 10, removing component");
            app.remove_component::<Count>(entity);
        }
    }
}

fn increment_count(app: &mut App) {
    for count in app.query::<&mut Count, ()>() {
        count.0 += 1;
    }
}

fn main() {
    App::with_preset(Headless).run(setup, update);
}
