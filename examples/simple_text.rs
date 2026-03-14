use comet::prelude::*;

fn setup(app: &mut App, renderer: &mut RenderHandle2D) {
    renderer.init_atlas();
    // Loading the font from the res/fonts directory with a rendered size of 77px
    renderer.load_font("./res/fonts/PublicPixel.ttf", 77.0);

    // Setting up camera
    app.spawn((
        Transform2D::new(), 
        Camera2D::new(v2::new(2.0, 2.0), 1.0, 1)
    ));

    // Creating the text entity
    app.spawn((
        Transform2D::new(),
        Text::new(
            "comet",                                // The content of the text
            "./res/fonts/PublicPixel.ttf", // The used font (right now exact to the font path)
            77.0,                                   // Pixel size at which the font will be drawn
            true,                                   // Should the text be visible
            sRgba::<f32>::from_hex("#abb2bfff"),    // Color of the text
        ),
    ));
}

fn update(app: &mut App, renderer: &mut RenderHandle2D, _dt: f32) {
    // Getting the window size (cached request)
    let size = renderer.size();

    // Recalculating the position of the text every frame to ensure the same relative position
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
