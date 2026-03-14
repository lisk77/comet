use comet::prelude::*;

fn setup(app: &mut App, renderer: &mut RenderHandle2D) {
    renderer.init_atlas();

    app.register_component::<Position2D>();
    app.register_component::<Color>();

    register_prefab!(
        app,
        "player",
        Position2D::from_vec(v2::new(0.0, 0.0)),
        Color::new(0.0, 1.0, 0.0, 1.0)
    );

    register_prefab!(
        app,
        "enemy",
        Position2D::from_vec(v2::new(5.0, 5.0)),
        Color::new(1.0, 0.0, 0.0, 1.0)
    );

    register_prefab!(
        app,
        "pickup",
        Position2D::from_vec(v2::new(-5.0, -5.0)),
        Color::new(1.0, 1.0, 0.0, 1.0)
    );

    if let Some(player_id) = app.spawn_prefab("player") {
        debug!("Spawned player with ID: {:?}", player_id);
    }

    if let Some(enemy_id) = app.spawn_prefab("enemy") {
        debug!("Spawned enemy with ID: {:?}", enemy_id);
    }

    if let Some(pickup_id) = app.spawn_prefab("pickup") {
        debug!("Spawned pickup with ID: {:?}", pickup_id);
    }
}

fn update(_app: &mut App, _renderer: &mut RenderHandle2D, _dt: f32) {}

fn main() {
    App::new()
        .with_title("Prefabs Example")
        .run::<Renderer2D>(setup, update);
}
