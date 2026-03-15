use comet::prelude::*;

fn setup(app: &mut App, renderer: &mut RenderHandle2D) {
    // Creating a texture atlas from the provided textures in the vector
    renderer.init_atlas_by_paths(vec!["res/textures/comet_icon.png".to_string()]);

    // Creating a camera entity
    app.spawn((Transform2D::new(), Camera2D::new(v2::new(2.0, 2.0), 1.0, 1)));

    // Creating a textured entity
    app.spawn((
        Transform2D::new(),
        Render2D::with_texture("res/textures/comet_icon.png"),
    ));
}

fn update(app: &mut App, renderer: &mut RenderHandle2D, _dt: f32) {
    renderer.render_scene_2d(app.scene_mut())
}

fn main() {
    App::new()
        .with_title("Textured Entity")
        .with_preset(App2D)
        .run::<Renderer2D>(setup, update);
}
