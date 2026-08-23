use comet::prelude::*;

fn setup(app: &mut App) {
    app.spawn(Camera2d::new());

    app.spawn((
        // This is the analog to `Transform` for something like HUDs
        ScreenPosition::new(Anchor::TopLeft).with_offset(v2::new(50.0, 100.0)),
        // For the renderer this is optional. It will be inferred as `TopLeft` and `Left` if missing
        TextLayout::new()
            .with_anchor(Anchor::TopLeft)
            .with_justification(TextJustification::Left),
        Text::new("comet", app.load("res://fonts/PublicPixel.ttf"))
            .with_font_size(77.0)
            .with_color(sRgba::<f32>::from_hex("#abb2bfff")),
    ));
}

fn update(_app: &mut App, _dt: f32) {}

fn main() {
    App::with_preset(App2D)
        .with_title("Simple Text")
        .run(setup, update);
}
