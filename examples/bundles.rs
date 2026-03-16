// This is the simple_move_2d example but using bundles
use comet::prelude::*;

#[derive(Component)]
struct Player;

bundle!(Camera {
    transform: Transform2D,
    camera: Camera2D
});

bundle!(Comet {
    player: Player,
    transform: Transform2D,
    render: Render2D
});

fn setup(app: &mut App, renderer: &mut RenderHandle2D) {
    renderer.init_atlas();

    app.register_component::<Player>();

    app.spawn_bundle(Camera {
        transform: Transform2D::new(),
        camera: Camera2D::new(v2::new(2.0, 2.0), 1.0, 1),
    });

    app.spawn_bundle(Comet {
        player: Player,
        transform: Transform2D::new(),
        render: Render2D::with_texture("res/textures/comet_icon.png"),
    });
}

fn update(app: &mut App, renderer: &mut RenderHandle2D, dt: f32) {
    handle_input(app, dt);

    renderer.render_scene_2d(app.scene_mut());
}

fn handle_input(app: &mut App, dt: f32) {
    let mut direction = v2::ZERO;
    if app.key_held(Key::KeyW) {
        direction += v2::Y;
    }
    if app.key_held(Key::KeyA) {
        direction -= v2::X;
    }
    if app.key_held(Key::KeyS) {
        direction -= v2::Y;
    }
    if app.key_held(Key::KeyD) {
        direction += v2::X;
    }

    if direction != v2::ZERO {
        app.query::<&mut Transform2D, With<Player>>().for_each(|t| {
            let normalized_dir = direction.normalize();
            let displacement = normalized_dir * 777.7 * dt;
            t.translate(displacement);
        });
    }
}

fn main() {
    App::new()
        .with_title("Simple Move 2D")
        .with_preset(App2D)
        .run::<Renderer2D>(setup, update);
}
