use comet::prelude::*;

fn setup(app: &mut App) {
    app.bind("jump", Key::Space);
    app.bind("jump", GamepadButton::South);

    app.bind("move_left",  Key::KeyA);
    app.bind("move_right", Key::KeyD);
    app.bind("move_left",  AxisBinding::new(GamepadAxis::LeftStickX, AxisDirection::Negative));
    app.bind("move_right", AxisBinding::new(GamepadAxis::LeftStickX, AxisDirection::Positive));

    app.bind("move_down", Key::KeyS);
    app.bind("move_up",   Key::KeyW);
    app.bind("move_down", AxisBinding::new(GamepadAxis::LeftStickY, AxisDirection::Negative));
    app.bind("move_up",   AxisBinding::new(GamepadAxis::LeftStickY, AxisDirection::Positive));
}

fn update(app: &mut App, _dt: f32) {
    if app.action_pressed("jump") {
        info!("Jump!");
    }

    let x = app.get_axis("move_left", "move_right");
    let y = app.get_axis("move_down", "move_up");

    if x != 0.0 || y != 0.0 {
        info!("Move ({:.2}, {:.2})", x, y);
    }
}

fn main() {
    App::with_preset(App2D)
        .run(setup, update);
}
