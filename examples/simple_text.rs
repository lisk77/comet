use comet::prelude::*;

fn setup(app: &mut App) {
    app.spawn(Camera2d::new());

    app.spawn((
        Transform::new(),
        Text::new("comet", app.load("res://fonts/PublicPixel.ttf"))
            .with_font_size(77.0)
            .with_color(sRgba::<f32>::from_hex("#abb2bfff")),
    ));
}

fn update(app: &mut App, _dt: f32) {
    let size = app.size();
    if size.width > 0 && size.height > 0 {
        text_update(app, v2::new(size.width as f32, size.height as f32));
    }
}

fn text_update(app: &mut App, size: v2) {
    if let Some((transform, _)) = app.query::<(&mut Transform, &Text), ()>().iter().next() {
        transform.set_x(-size.x() * 0.5 + 50.0);
        transform.set_y(size.y() * 0.5 - 100.0);
    }
}

fn main() {
    App::with_preset(App2D)
        .with_title("Simple Text")
        .run(setup, update);
}
