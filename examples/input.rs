use comet::prelude::*;

fn setup(_app: &mut App) {}

fn update(app: &mut App, _dt: f32) {
    for key in [
        Key::KeyA, Key::KeyB, Key::KeyC, Key::KeyD, Key::KeyE,
        Key::KeyF, Key::KeyG, Key::KeyH, Key::KeyI, Key::KeyJ,
        Key::KeyK, Key::KeyL, Key::KeyM, Key::KeyN, Key::KeyO,
        Key::KeyP, Key::KeyQ, Key::KeyR, Key::KeyS, Key::KeyT,
        Key::KeyU, Key::KeyV, Key::KeyW, Key::KeyX, Key::KeyY,
        Key::KeyZ, Key::Space, Key::Enter, Key::Escape,
        Key::ArrowUp, Key::ArrowDown, Key::ArrowLeft, Key::ArrowRight,
    ] {
        if app.key_pressed(key) {
            info!("Pressed:  {:?}", key);
        }
        if app.key_released(key) {
            info!("Released: {:?}", key);
        }
    }

    if app.modifier_held(Modifier::Shift) { info!("Modifier held: Shift"); }
    if app.modifier_held(Modifier::Ctrl)  { info!("Modifier held: Ctrl"); }
    if app.modifier_held(Modifier::Alt)   { info!("Modifier held: Alt"); }

    for button in [Button::Left, Button::Right, Button::Middle] {
        if app.mouse_pressed(button)  { info!("Mouse pressed:  {:?}", button); }
        if app.mouse_released(button) { info!("Mouse released: {:?}", button); }
    }

    if app.mouse_moved() {
        let (x, y) = app.mouse_position();
        info!("Mouse position: ({:.1}, {:.1})", x, y);
    }

    let (dx, dy) = app.mouse_delta();
    if dx != 0.0 || dy != 0.0 {
        info!("Mouse delta: ({:.1}, {:.1})", dx, dy);
    }

    let (sx, sy) = app.scroll_delta();
    if sx != 0.0 || sy != 0.0 {
        info!("Scroll delta: ({:.1}, {:.1})", sx, sy);
    }

    if app.cursor_entered() { info!("Cursor entered window"); }
    if app.cursor_exited()  { info!("Cursor exited window"); }

    for id in app.connected_gamepads() {
        for button in [
            GamepadButton::South, GamepadButton::East,
            GamepadButton::North, GamepadButton::West,
            GamepadButton::LeftTrigger, GamepadButton::RightTrigger,
            GamepadButton::LeftTrigger2, GamepadButton::RightTrigger2,
            GamepadButton::Select, GamepadButton::Start,
            GamepadButton::DPadUp, GamepadButton::DPadDown,
            GamepadButton::DPadLeft, GamepadButton::DPadRight,
        ] {
            if app.gamepad_button_pressed(id, button)  { info!("Gamepad {:?} pressed:  {:?}", id, button); }
            if app.gamepad_button_released(id, button) { info!("Gamepad {:?} released: {:?}", id, button); }
        }

        for axis in [
            GamepadAxis::LeftStickX, GamepadAxis::LeftStickY,
            GamepadAxis::RightStickX, GamepadAxis::RightStickY,
            GamepadAxis::LeftZ, GamepadAxis::RightZ,
        ] {
            let value = app.gamepad_axis(id, axis);
            if value.abs() > 0.1 {
                info!("Gamepad {:?} axis {:?}: {:.2}", id, axis, value);
            }
        }
    }
}

fn main() {
    App::with_preset(App2D)
        .run(setup, update);
}
