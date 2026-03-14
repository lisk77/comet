use comet::prelude::*;

#[derive(Component)]
struct Count(u8);

fn setup(app: &mut App, _renderer: &mut RenderHandle2D) {
    app.register_component::<Count>();

    app.spawn(Count::new());
}

fn update(app: &mut App, _renderer: &mut RenderHandle2D, _dt: f32) {
    // Note: the setup tick and first update tick are handled as the same tick
    if app.query::<&Count, Added<Count>>().iter().count() != 0 {
        info!("Count was added this tick");
    }

    counter(app);

    if app.query::<&Count, Changed<Count>>().iter().count() != 0 {
        info!(
            "Count was changed to {}",
            app.query::<&Count, ()>().iter().next().unwrap().0
        )
    }

    match app.query::<(Entity, &Count), ()>().iter().next() {
        Some((e, c)) => {
            if c.0 == 10 {
                info!("Count reached 10, removing component");
                app.remove_component::<Count>(e);
            }
        }
        None => {}
    }
}

fn counter(app: &mut App) {
    app.query::<&mut Count, ()>().for_each(|c| {
        c.0 += 1;
    });
}

fn main() {
    App::new()
        .with_title("Query Change Filters")
        .run::<Renderer2D>(setup, update);
}
