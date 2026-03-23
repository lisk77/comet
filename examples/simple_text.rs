use comet::prelude::*;

fn setup(app: &mut App, renderer: &mut RenderHandle2D) {
    renderer.init_atlas();

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

    app.spawn((Transform2D::new(), Camera2D::new(v2::new(2.0, 2.0), 1.0, 1)));
}

fn update(app: &mut App, renderer: &mut RenderHandle2D, _dt: f32) {
    let size = renderer.size();

    if size.width > 0 && size.height > 0 {
        let mut text_query = app.query::<(&mut Transform2D, &Text), ()>().iter();
        if let Some((transform, _)) = text_query.next() {
            transform.position_mut().set_x(-((size.width - 50) as f32));
            transform.position_mut().set_y((size.height - 100) as f32);
        }
    }

    renderer.render_scene_2d(app.scene_mut());
}

fn main() {
    App::new()
        .with_preset(App2D)
        .with_title("Simple Text")
        .run::<Renderer2D>(setup, update);
}
