use comet::prelude::*;

#[derive(Component)]
struct Hitbox {
    dimensions: v3,
}

impl Gizmo for Hitbox {
    fn draw_gizmo(&self, position: v3, _rotation: v3, _scale: v3, buffer: &mut GizmoBuffer) {
        buffer.draw_rect(position, self.dimensions, LinearRgba::new(0.0, 1.0, 0.0, 1.0));
    }
}

fn setup(app: &mut App) {
    app.register_component::<Hitbox>();

    app.spawn_bundle(Camera2d::new(1.0, 1));

    let e = app.spawn((
        Transform::with_position(v3::new(0.0, 0.0, 0.0)),
        Hitbox { dimensions: v3::new(64.0, 64.0, 0.0) },
        Sprite::with_texture("res://textures/comet-64.png"),  
    ));

    app.show_gizmo::<Hitbox>(e);
}

fn update(app: &mut App, dt: f32) {
    handle_input(app, dt);
    let (entity, _) = app.query::<(Entity, &Hitbox), ()>().iter().next().unwrap();
    app.show_gizmo::<Hitbox>(entity);
}

fn handle_input(app: &mut App, dt: f32) {
    let mut direction = v2::ZERO;
    if app.key_held(Key::KeyW) { direction += v2::Y; }
    if app.key_held(Key::KeyA) { direction -= v2::X; }
    if app.key_held(Key::KeyS) { direction -= v2::Y; }
    if app.key_held(Key::KeyD) { direction += v2::X; }

    if direction != v2::ZERO {
        app.query::<&mut Transform, With<Hitbox>>().for_each(|t| {
            let normalized_dir = direction.normalize();
            let displacement = normalized_dir * 777.7 * dt;
            t.translate(displacement.into());
        });
    }
}

fn main() {
    App::with_preset(App2D)
        .with_title("Gizmos")
        .run(setup, update);
}
