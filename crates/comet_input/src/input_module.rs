use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use comet_app::{App, Module};
use comet_macros::module;
use comet_window::WinitModule;
use gilrs::{Axis, Button as GilrsButton, EventType, Gilrs};
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

pub use crate::keyboard::Key;
pub use crate::mouse::Button;
pub use gilrs::Button as GamepadButton;
pub use gilrs::Axis as GamepadAxis;
pub use gilrs::GamepadId;

pub enum Modifier {
    Shift,
    Ctrl,
    Alt,
    Super,
}

enum RawInputEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    ModifiersChanged(ModifiersState),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    MouseMoved(f32, f32),
    MouseDelta(f32, f32),
    MouseScrolled(f32, f32),
    CursorEntered,
    CursorLeft,
}

struct GamepadSnapshot {
    buttons_pressed: HashSet<GilrsButton>,
    buttons_held: HashSet<GilrsButton>,
    buttons_released: HashSet<GilrsButton>,
    axes: HashMap<Axis, f32>,
}

impl GamepadSnapshot {
    fn new() -> Self {
        Self {
            buttons_pressed: HashSet::new(),
            buttons_held: HashSet::new(),
            buttons_released: HashSet::new(),
            axes: HashMap::new(),
        }
    }

    fn clear_transient(&mut self) {
        self.buttons_pressed.clear();
        self.buttons_released.clear();
    }
}

struct InputSnapshot {
    keys_pressed: HashSet<KeyCode>,
    keys_held: HashSet<KeyCode>,
    keys_released: HashSet<KeyCode>,
    modifiers: ModifiersState,
    mouse_pressed: HashSet<MouseButton>,
    mouse_held: HashSet<MouseButton>,
    mouse_released: HashSet<MouseButton>,
    mouse_position: (f32, f32),
    mouse_delta: (f32, f32),
    mouse_moved: bool,
    scroll_delta: (f32, f32),
    cursor_entered: bool,
    cursor_exited: bool,
    cursor_in_window: bool,
    gamepads: HashMap<GamepadId, GamepadSnapshot>,
}

impl InputSnapshot {
    fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_held: HashSet::new(),
            keys_released: HashSet::new(),
            modifiers: ModifiersState::empty(),
            mouse_pressed: HashSet::new(),
            mouse_held: HashSet::new(),
            mouse_released: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            mouse_moved: false,
            scroll_delta: (0.0, 0.0),
            cursor_entered: false,
            cursor_exited: false,
            cursor_in_window: false,
            gamepads: HashMap::new(),
        }
    }
}

pub struct InputModule {
    queue: Arc<Mutex<Vec<RawInputEvent>>>,
    snapshot: InputSnapshot,
    gilrs: Gilrs,
}

impl InputModule {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            snapshot: InputSnapshot::new(),
            gilrs: Gilrs::new().unwrap(),
        }
    }

    fn advance_tick(&mut self) {
        self.snapshot.keys_pressed.clear();
        self.snapshot.keys_released.clear();
        self.snapshot.mouse_pressed.clear();
        self.snapshot.mouse_released.clear();
        self.snapshot.mouse_delta = (0.0, 0.0);
        self.snapshot.mouse_moved = false;
        self.snapshot.scroll_delta = (0.0, 0.0);
        self.snapshot.cursor_entered = false;
        self.snapshot.cursor_exited = false;

        for gp in self.snapshot.gamepads.values_mut() {
            gp.clear_transient();
        }

        // drain winit events
        let events: Vec<RawInputEvent> = self.queue.lock().unwrap().drain(..).collect();
        for event in events {
            match event {
                RawInputEvent::KeyPressed(k) => {
                    if self.snapshot.keys_held.insert(k) {
                        self.snapshot.keys_pressed.insert(k);
                    }
                }
                RawInputEvent::KeyReleased(k) => {
                    self.snapshot.keys_held.remove(&k);
                    self.snapshot.keys_released.insert(k);
                }
                RawInputEvent::ModifiersChanged(state) => {
                    self.snapshot.modifiers = state;
                }
                RawInputEvent::MousePressed(b) => {
                    if self.snapshot.mouse_held.insert(b) {
                        self.snapshot.mouse_pressed.insert(b);
                    }
                }
                RawInputEvent::MouseReleased(b) => {
                    self.snapshot.mouse_held.remove(&b);
                    self.snapshot.mouse_released.insert(b);
                }
                RawInputEvent::MouseMoved(x, y) => {
                    self.snapshot.mouse_position = (x, y);
                    self.snapshot.mouse_moved = true;
                }
                RawInputEvent::MouseDelta(dx, dy) => {
                    self.snapshot.mouse_delta.0 += dx;
                    self.snapshot.mouse_delta.1 += dy;
                }
                RawInputEvent::MouseScrolled(x, y) => {
                    self.snapshot.scroll_delta.0 += x;
                    self.snapshot.scroll_delta.1 += y;
                }
                RawInputEvent::CursorEntered => {
                    self.snapshot.cursor_in_window = true;
                    self.snapshot.cursor_entered = true;
                }
                RawInputEvent::CursorLeft => {
                    self.snapshot.cursor_in_window = false;
                    self.snapshot.cursor_exited = true;
                }
            }
        }

        while let Some(gilrs::Event { id, event, .. }) = self.gilrs.next_event() {
            let gp = self.snapshot.gamepads.entry(id).or_insert_with(GamepadSnapshot::new);
            match event {
                EventType::ButtonPressed(btn, _) => {
                    if gp.buttons_held.insert(btn) {
                        gp.buttons_pressed.insert(btn);
                    }
                }
                EventType::ButtonReleased(btn, _) => {
                    gp.buttons_held.remove(&btn);
                    gp.buttons_released.insert(btn);
                }
                EventType::AxisChanged(axis, value, _) => {
                    gp.axes.insert(axis, value);
                }
                EventType::Disconnected => {
                    self.snapshot.gamepads.remove(&id);
                }
                _ => {}
            }
        }
    }
}

fn input_pre_tick(app: &mut App) {
    let mut module = app.take_module::<InputModule>().unwrap();
    module.advance_tick();
    app.reinsert_module(module);
}

#[module]
impl InputModule {
    pub fn key_pressed(&self, key: Key) -> bool {
        self.snapshot.keys_pressed.contains(&key)
    }

    pub fn key_held(&self, key: Key) -> bool {
        self.snapshot.keys_held.contains(&key)
    }

    pub fn key_released(&self, key: Key) -> bool {
        self.snapshot.keys_released.contains(&key)
    }

    pub fn any_key_pressed(&self) -> bool {
        !self.snapshot.keys_pressed.is_empty()
    }

    pub fn modifier_held(&self, modifier: Modifier) -> bool {
        match modifier {
            Modifier::Shift => self.snapshot.modifiers.shift_key(),
            Modifier::Ctrl => self.snapshot.modifiers.control_key(),
            Modifier::Alt => self.snapshot.modifiers.alt_key(),
            Modifier::Super => self.snapshot.modifiers.super_key(),
        }
    }

    pub fn mouse_pressed(&self, button: Button) -> bool {
        self.snapshot.mouse_pressed.contains(&button)
    }

    pub fn mouse_held(&self, button: Button) -> bool {
        self.snapshot.mouse_held.contains(&button)
    }

    pub fn mouse_released(&self, button: Button) -> bool {
        self.snapshot.mouse_released.contains(&button)
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        self.snapshot.mouse_position
    }

    pub fn mouse_delta(&self) -> (f32, f32) {
        self.snapshot.mouse_delta
    }

    pub fn mouse_moved(&self) -> bool {
        self.snapshot.mouse_moved
    }

    pub fn scroll_delta(&self) -> (f32, f32) {
        self.snapshot.scroll_delta
    }

    pub fn cursor_entered(&self) -> bool {
        self.snapshot.cursor_entered
    }

    pub fn cursor_exited(&self) -> bool {
        self.snapshot.cursor_exited
    }

    pub fn cursor_in_window(&self) -> bool {
        self.snapshot.cursor_in_window
    }

    pub fn gamepad_button_pressed(&self, id: GamepadId, button: GamepadButton) -> bool {
        self.snapshot.gamepads.get(&id).map_or(false, |gp| gp.buttons_pressed.contains(&button))
    }

    pub fn gamepad_button_held(&self, id: GamepadId, button: GamepadButton) -> bool {
        self.snapshot.gamepads.get(&id).map_or(false, |gp| gp.buttons_held.contains(&button))
    }

    pub fn gamepad_button_released(&self, id: GamepadId, button: GamepadButton) -> bool {
        self.snapshot.gamepads.get(&id).map_or(false, |gp| gp.buttons_released.contains(&button))
    }

    pub fn gamepad_axis(&self, id: GamepadId, axis: GamepadAxis) -> f32 {
        self.snapshot.gamepads.get(&id).and_then(|gp| gp.axes.get(&axis)).copied().unwrap_or(0.0)
    }

    pub fn connected_gamepads(&self) -> Vec<GamepadId> {
        self.snapshot.gamepads.keys().copied().collect()
    }
}

impl Module for InputModule {
    fn dependencies(app: &mut App)
    where
        Self: Sized,
    {
        if !app.has_module::<WinitModule>() {
            app.add_module(WinitModule::new());
        }
    }

    fn build(&mut self, app: &mut App) {
        let queue = Arc::clone(&self.queue);
        app.get_module_mut::<WinitModule>().add_event_hook(move |event| {
            let raw: Option<RawInputEvent> = match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        if let PhysicalKey::Code(keycode) = key_event.physical_key {
                            Some(match key_event.state {
                                ElementState::Pressed => RawInputEvent::KeyPressed(keycode),
                                ElementState::Released => RawInputEvent::KeyReleased(keycode),
                            })
                        } else {
                            None
                        }
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        Some(RawInputEvent::ModifiersChanged(modifiers.state()))
                    }
                    WindowEvent::MouseInput { state, button, .. } => Some(match state {
                        ElementState::Pressed => RawInputEvent::MousePressed(*button),
                        ElementState::Released => RawInputEvent::MouseReleased(*button),
                    }),
                    WindowEvent::CursorMoved { position, .. } => {
                        Some(RawInputEvent::MouseMoved(position.x as f32, position.y as f32))
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let (x, y) = match delta {
                            MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                            MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
                        };
                        Some(RawInputEvent::MouseScrolled(x, y))
                    }
                    WindowEvent::CursorEntered { .. } => Some(RawInputEvent::CursorEntered),
                    WindowEvent::CursorLeft { .. } => Some(RawInputEvent::CursorLeft),
                    _ => None,
                },
                Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                    Some(RawInputEvent::MouseDelta(delta.0 as f32, delta.1 as f32))
                }
                _ => None,
            };
            if let Some(e) = raw {
                if let Ok(mut q) = queue.lock() {
                    q.push(e);
                }
            }
        });

        app.add_pre_tick_hook(input_pre_tick);
    }
}
