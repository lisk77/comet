use comet::prelude::*;

// Defines typed keybind names
#[derive(Action)]
enum GameAction {
    Jump,
    MoveLeft,
    MoveRight,
    MoveDown,
    MoveUp,
    Copy,
    Paste,
}

fn setup(app: &mut App) {
    app.bind(GameAction::Jump, Key::Space);
    app.bind(GameAction::Jump, GamepadButton::South);

    app.bind(GameAction::MoveLeft, Key::KeyA);
    app.bind(GameAction::MoveRight, Key::KeyD);
    app.bind(
        GameAction::MoveLeft,
        AxisBinding::new(GamepadAxis::LeftStickX, AxisDirection::Negative),
    );
    app.bind(
        GameAction::MoveRight,
        AxisBinding::new(GamepadAxis::LeftStickX, AxisDirection::Positive),
    );

    app.bind(GameAction::MoveDown, Key::KeyS);
    app.bind(GameAction::MoveUp, Key::KeyW);
    app.bind(
        GameAction::MoveDown,
        AxisBinding::new(GamepadAxis::LeftStickY, AxisDirection::Negative),
    );
    app.bind(
        GameAction::MoveUp,
        AxisBinding::new(GamepadAxis::LeftStickY, AxisDirection::Positive),
    );

    app.bind(GameAction::Copy, (Key::ControlLeft, Key::KeyC));
    app.bind(GameAction::Paste, (Key::ControlLeft, Key::KeyV));
}

fn update(app: &mut App, _dt: f32) {
    if app.action_pressed(GameAction::Jump) {
        info!("Jump!");
    }

    let x = app.axis(GameAction::MoveLeft, GameAction::MoveRight);
    let y = app.axis(GameAction::MoveDown, GameAction::MoveUp);

    if x != 0.0 || y != 0.0 {
        info!("Move ({:.2}, {:.2})", x, y);
    }

    if app.action_pressed(GameAction::Copy) {
        info!("Copy!");
    }
    if app.action_pressed(GameAction::Paste) {
        info!("Paste!");
    }
}

fn main() {
    App::with_preset(App2D)
        .with_title("Input Mapping")
        .run(setup, update);
}
