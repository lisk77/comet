use comet::prelude::*;

fn setup(app: &mut App, renderer: &mut Renderer2D) {
    renderer.init_atlas();
    // Loading the font from the res/fonts directory with a rendered size of 77px
    renderer.load_font("./res/fonts/PressStart2P-Regular.ttf", 77.0);

    // Setting up camera
    let camera = app.new_entity();

    app.add_component(camera, Transform2D::new());
    app.add_component(camera, Camera2D::new(v2::new(2.0, 2.0), 1.0, 1));

    // Creating the text entity
    let text = app.new_entity();
    app.add_component(text, Transform2D::new());
    app.add_component(
        text,
        Text::new(
            "comet",                                // The content of the text
            "./res/fonts/PressStart2P-Regular.ttf", // The used font (right now exact to the font path)
            77.0,                                   // Pixel size at which the font will be drawn
            true,                                   // Should the text be visible
            sRgba::<f32>::from_hex("#abb2bfff"),    // Color of the text
        ),
    );
}

#[allow(unused_variables)]
fn update(app: &mut App, renderer: &mut Renderer2D, dt: f32) {
    // Getting the windows size
    let size = renderer.size();

    // Recalculating the position of the text every frame to ensure the same relative position
    let transform = app.get_component_mut::<Transform2D>(1).unwrap();
    transform.position_mut().set_x(-((size.width - 50) as f32));
    transform.position_mut().set_y((size.height - 100) as f32);

    renderer.render_scene_2d(app.scene_mut());
}

fn main() {
    App::new()
        .with_preset(App2D)
        .with_title("Simple Text")
        .run::<Renderer2D>(setup, update);
}
