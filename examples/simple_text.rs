use comet::prelude::*;

fn setup(app: &mut App, _renderer: &mut RenderHandle2D) {
    app.spawn((Transform2D::new(), Camera2D::new(v2::new(2.0, 2.0), 1.0, 1)));

    app.spawn((
        Transform2D::new(),
        Text::new(
            "comet",
            app.load("res://fonts/PublicPixel.ttf"),
            77.0,
            true,
            sRgba::<f32>::from_hex("#abb2bfff"),
        ),
    ));
}

fn update(app: &mut App, renderer: &mut RenderHandle2D, _dt: f32) {
    let size = renderer.size();

    if size.width > 0 && size.height > 0 {
        text_update(app, v2::new(size.width as f32, size.height as f32));
    }

    renderer.render_scene_2d(app.scene_mut());
}

fn text_update(app: &mut App, size: v2) {
    if let Some((transform, _)) = app.query::<(&mut Transform2D, &Text), ()>().iter().next() {
        transform.position_mut().set_x(-((size.x() - 50.0) as f32));
        transform.position_mut().set_y((size.y() - 100.0) as f32);
    }
}

fn main() {
    App::with_preset(App2D)
        .with_title("Simple Text")
        .run::<Renderer2D>(setup, update);
}
