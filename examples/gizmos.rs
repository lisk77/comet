use comet::prelude::*;

#[derive(Component)]
struct Hitbox {
    dimensions: v3,
}

impl Gizmo for Hitbox {
    fn draw_gizmo(
        &self,
        position: v3,
        _rotation: EulerAngles,
        _scale: v3,
        buffer: &mut GizmoBuffer,
    ) {
        buffer.draw_rect(
            position,
            self.dimensions,
            LinearRgba::new(0.0, 1.0, 0.0, 1.0),
        );
    }
}

fn setup(app: &mut App) {
    app.spawn(Camera2d);
    let entity = app.spawn((
        Transform::new(),
        Hitbox {
            dimensions: v3::new(64.0, 64.0, 0.0),
        },
        Sprite::with_texture("res://textures/comet-64.png"),
    ));
    app.show_gizmo::<Hitbox>(entity);
}

fn update(app: &mut App, dt: f32) {
    move_hitboxes(
        app.query::<&mut Transform, With<Hitbox>>(),
        movement_direction(app),
        dt,
    );

    if let Some((entity, _)) = app.query::<(Entity, &Hitbox), ()>().next() {
        app.show_gizmo::<Hitbox>(entity);
    }
}

fn movement_direction(app: &App) -> v2 {
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
    direction
}

fn move_hitboxes(hitboxes: Query<&mut Transform, With<Hitbox>>, direction: v2, dt: f32) {
    if direction == v2::ZERO {
        return;
    }

    let displacement = direction.normalize() * 777.7 * dt;
    for transform in hitboxes {
        transform.translate(displacement.into());
    }
}

fn main() {
    App::with_preset(App2D)
        .with_title("Gizmos")
        .run(setup, update);
}
