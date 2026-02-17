use comet::prelude::*;

bundle!(BallBundle {
    transform: Transform2D,
    render: Render2D,
});

#[allow(unused_variables)]
fn setup(app: &mut App, renderer: &mut RenderHandle2D) {
    app.register_component::<Transform2D>();
    app.register_component::<Render2D>();

    app.spawn_bundle(BallBundle {
        transform: Transform2D::new(),
        render: Render2D::with_texture("examples/assets/ball.png"),
    });
}

#[allow(unused_variables)]
fn update(app: &mut App, renderer: &mut RenderHandle2D, dt: f32) {}

fn main() {
    App::new()
        .with_title("Bundles")
        .run::<Renderer2D>(setup, update);
}
