use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use comet_app::{App, Module};
use comet_macros::module;
use comet_window::WinitModule;
use gilrs::{Axis, Button as GilrsButton, EventType, Gilrs};
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub use crate::keyboard::Key;
pub use crate::mouse::Button;
pub use gilrs::Button as GamepadButton;
pub use gilrs::Axis as GamepadAxis;
pub use gilrs::GamepadId;

const DEFAULT_DEADZONE: f32 = 0.2;

pub enum AxisDirection {
    Positive,
    Negative,
}

pub struct AxisBinding {
    pub axis: GamepadAxis,
    pub direction: AxisDirection,
}

impl AxisBinding {
    pub fn new(axis: GamepadAxis, direction: AxisDirection) -> Self {
        Self { axis, direction }
    }
}

pub trait Binding: Send + 'static {
    fn strength(&self, state: &InputState) -> f32;

    fn pressed(&self, state: &InputState) -> bool {
        self.strength(state) > DEFAULT_DEADZONE
    }

    fn held(&self, state: &InputState) -> bool {
        self.strength(state) > DEFAULT_DEADZONE
    }

    fn released(&self, state: &InputState) -> bool;
}

impl Binding for Key {
    fn strength(&self, state: &InputState) -> f32 {
        if state.keys_held.contains(self) { 1.0 } else { 0.0 }
    }

    fn pressed(&self, state: &InputState) -> bool { state.keys_pressed.contains(self) }
    fn held(&self, state: &InputState) -> bool { state.keys_held.contains(self) }
    fn released(&self, state: &InputState) -> bool { state.keys_released.contains(self) }
}

impl Binding for Button {
    fn strength(&self, state: &InputState) -> f32 {
        if state.mouse_held.contains(self) { 1.0 } else { 0.0 }
    }
    
    fn pressed(&self, state: &InputState) -> bool { state.mouse_pressed.contains(self) }
    fn held(&self, state: &InputState) -> bool { state.mouse_held.contains(self) }
    fn released(&self, state: &InputState) -> bool { state.mouse_released.contains(self) }
}

impl Binding for GamepadButton {
    fn strength(&self, state: &InputState) -> f32 {
        if state.gamepads.values().any(|gp| gp.buttons_held.contains(self)) { 1.0 } else { 0.0 }
    }

    fn pressed(&self, state: &InputState) -> bool {
        state.gamepads.values().any(|gp| gp.buttons_pressed.contains(self))
    }

    fn held(&self, state: &InputState) -> bool {
        state.gamepads.values().any(|gp| gp.buttons_held.contains(self))
    }

    fn released(&self, state: &InputState) -> bool {
        state.gamepads.values().any(|gp| gp.buttons_released.contains(self))
    }
}

impl Binding for AxisBinding {
    fn strength(&self, state: &InputState) -> f32 {
        let raw = state.gamepads.values()
            .filter_map(|gp| gp.axes.get(&self.axis).copied())
            .fold(0.0f32, |a, b| if b.abs() > a.abs() { b } else { a });
        let value = match self.direction {
            AxisDirection::Positive => raw.max(0.0),
            AxisDirection::Negative => (-raw).max(0.0),
        };
        if value < DEFAULT_DEADZONE { 0.0 } else { value }
    }

    fn released(&self, state: &InputState) -> bool {
        self.strength(state) <= DEFAULT_DEADZONE
    }
}

macro_rules! impl_binding_tuple {
    ($($T:ident),+) => {
        impl<$($T: Binding),+> Binding for ($($T,)+) {
            fn strength(&self, state: &InputState) -> f32 {
                #[allow(non_snake_case)]
                let ($($T,)+) = self;
                if [$($T.strength(state)),+].iter().all(|&s| s > DEFAULT_DEADZONE) { 1.0 } else { 0.0 }
            }

            fn pressed(&self, state: &InputState) -> bool {
                #[allow(non_snake_case)]
                let ($($T,)+) = self;
                [$($T.strength(state)),+].iter().all(|&s| s > DEFAULT_DEADZONE)
                    && [$($T.pressed(state)),+].iter().any(|&p| p)
            }

            fn released(&self, state: &InputState) -> bool {
                !self.held(state)
            }
        }
    };
}

impl_binding_tuple!(A, B);
impl_binding_tuple!(A, B, C);
impl_binding_tuple!(A, B, C, D);

pub struct InputMap {
    bindings: HashMap<String, Vec<Box<dyn Binding>>>,
}

impl InputMap {
    fn new() -> Self {
        Self { bindings: HashMap::new() }
    }

    fn bind(&mut self, action: impl Into<String>, binding: impl Binding) {
        self.bindings.entry(action.into()).or_default().push(Box::new(binding));
    }

    fn unbind(&mut self, action: &str) {
        self.bindings.remove(action);
    }
}

enum RawInputEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    MouseMoved(f32, f32),
    MouseDelta(f32, f32),
    MouseScrolled(f32, f32),
    CursorEntered,
    CursorLeft,
    FocusLost,
}

pub struct GamepadState {
    pub buttons_pressed: HashSet<GilrsButton>,
    pub buttons_held: HashSet<GilrsButton>,
    pub buttons_released: HashSet<GilrsButton>,
    pub axes: HashMap<Axis, f32>,
}

impl GamepadState {
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

pub struct InputState {
    pub keys_pressed: HashSet<KeyCode>,
    pub keys_held: HashSet<KeyCode>,
    pub keys_released: HashSet<KeyCode>,
    pub mouse_pressed: HashSet<MouseButton>,
    pub mouse_held: HashSet<MouseButton>,
    pub mouse_released: HashSet<MouseButton>,
    pub mouse_position: (f32, f32),
    pub mouse_delta: (f32, f32),
    pub mouse_moved: bool,
    pub scroll_delta: (f32, f32),
    pub cursor_entered: bool,
    pub cursor_exited: bool,
    pub cursor_in_window: bool,
    pub gamepads: HashMap<GamepadId, GamepadState>,
}

impl InputState {
    fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_held: HashSet::new(),
            keys_released: HashSet::new(),
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
    frame_gen: Arc<AtomicU64>,
    last_gen: u64,
    state: InputState,
    gilrs: Gilrs,
    input_map: InputMap,
}

impl InputModule {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            frame_gen: Arc::new(AtomicU64::new(0)),
            last_gen: u64::MAX,
            state: InputState::new(),
            gilrs: Gilrs::new().unwrap(),
            input_map: InputMap::new(),
        }
    }

    fn advance_tick(&mut self) {
        let current_gen = self.frame_gen.load(Ordering::Relaxed);
        let new_frame = current_gen != self.last_gen;

        if new_frame {
            self.last_gen = current_gen;
            self.state.keys_pressed.clear();
            self.state.keys_released.clear();
            self.state.mouse_pressed.clear();
            self.state.mouse_released.clear();
            self.state.mouse_delta = (0.0, 0.0);
            self.state.mouse_moved = false;
            self.state.scroll_delta = (0.0, 0.0);
            self.state.cursor_entered = false;
            self.state.cursor_exited = false;
            for gp in self.state.gamepads.values_mut() {
                gp.clear_transient();
            }
        }

        let events: Vec<RawInputEvent> = self.queue.lock().unwrap().drain(..).collect();
        for event in events {
            match event {
                RawInputEvent::KeyPressed(k) => {
                    if self.state.keys_held.insert(k) {
                        self.state.keys_pressed.insert(k);
                    }
                }
                RawInputEvent::KeyReleased(k) => {
                    self.state.keys_held.remove(&k);
                    self.state.keys_released.insert(k);
                }
                RawInputEvent::MousePressed(b) => {
                    if self.state.mouse_held.insert(b) {
                        self.state.mouse_pressed.insert(b);
                    }
                }
                RawInputEvent::MouseReleased(b) => {
                    self.state.mouse_held.remove(&b);
                    self.state.mouse_released.insert(b);
                }
                RawInputEvent::CursorEntered => {
                    self.state.cursor_in_window = true;
                    self.state.cursor_entered = true;
                }
                RawInputEvent::CursorLeft => {
                    self.state.cursor_in_window = false;
                    self.state.cursor_exited = true;
                }
                RawInputEvent::MouseMoved(x, y) => {
                    self.state.mouse_position = (x, y);
                    self.state.mouse_moved = true;
                }
                RawInputEvent::MouseDelta(dx, dy) => {
                    self.state.mouse_delta.0 += dx;
                    self.state.mouse_delta.1 += dy;
                }
                RawInputEvent::MouseScrolled(x, y) => {
                    self.state.scroll_delta.0 += x;
                    self.state.scroll_delta.1 += y;
                }
                RawInputEvent::FocusLost => {
                    self.state.keys_held.clear();
                    self.state.mouse_held.clear();
                    for gp in self.state.gamepads.values_mut() {
                        gp.buttons_held.clear();
                        gp.axes.clear();
                    }
                }
            }
        }

        while let Some(gilrs::Event { id, event, .. }) = self.gilrs.next_event() {
            let gp = self.state.gamepads.entry(id).or_insert_with(GamepadState::new);
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
                    self.state.gamepads.remove(&id);
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
        self.state.keys_pressed.contains(&key)
    }

    pub fn key_held(&self, key: Key) -> bool {
        self.state.keys_held.contains(&key)
    }

    pub fn key_released(&self, key: Key) -> bool {
        self.state.keys_released.contains(&key)
    }

    pub fn any_key_pressed(&self) -> bool {
        !self.state.keys_pressed.is_empty()
    }

    pub fn mouse_pressed(&self, button: Button) -> bool {
        self.state.mouse_pressed.contains(&button)
    }

    pub fn mouse_held(&self, button: Button) -> bool {
        self.state.mouse_held.contains(&button)
    }

    pub fn mouse_released(&self, button: Button) -> bool {
        self.state.mouse_released.contains(&button)
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        self.state.mouse_position
    }

    pub fn mouse_delta(&self) -> (f32, f32) {
        self.state.mouse_delta
    }

    pub fn mouse_moved(&self) -> bool {
        self.state.mouse_moved
    }

    pub fn scroll_delta(&self) -> (f32, f32) {
        self.state.scroll_delta
    }

    pub fn cursor_entered(&self) -> bool {
        self.state.cursor_entered
    }

    pub fn cursor_exited(&self) -> bool {
        self.state.cursor_exited
    }

    pub fn cursor_in_window(&self) -> bool {
        self.state.cursor_in_window
    }

    pub fn gamepad_button_pressed(&self, id: GamepadId, button: GamepadButton) -> bool {
        self.state.gamepads.get(&id).map_or(false, |gp| gp.buttons_pressed.contains(&button))
    }

    pub fn gamepad_button_held(&self, id: GamepadId, button: GamepadButton) -> bool {
        self.state.gamepads.get(&id).map_or(false, |gp| gp.buttons_held.contains(&button))
    }

    pub fn gamepad_button_released(&self, id: GamepadId, button: GamepadButton) -> bool {
        self.state.gamepads.get(&id).map_or(false, |gp| gp.buttons_released.contains(&button))
    }

    pub fn gamepad_axis(&self, id: GamepadId, axis: GamepadAxis) -> f32 {
        self.state.gamepads.get(&id).and_then(|gp| gp.axes.get(&axis)).copied().unwrap_or(0.0)
    }

    pub fn connected_gamepads(&self) -> Vec<GamepadId> {
        self.state.gamepads.keys().copied().collect()
    }

    pub fn bind(&mut self, action: impl Into<String>, binding: impl Binding) {
        self.input_map.bind(action, binding);
    }

    pub fn unbind(&mut self, action: impl Into<String>) {
        self.input_map.unbind(&action.into());
    }

    pub fn action_strength(&self, action: &str) -> f32 {
        self.input_map.bindings.get(action)
            .map_or(0.0, |bs| bs.iter().map(|b| b.strength(&self.state)).fold(0.0f32, f32::max))
    }

    pub fn get_axis(&self, negative: &str, positive: &str) -> f32 {
        self.action_strength(positive) - self.action_strength(negative)
    }

    pub fn action_pressed(&self, action: &str) -> bool {
        self.input_map.bindings.get(action)
            .map_or(false, |bs| bs.iter().any(|b| b.pressed(&self.state)))
    }

    pub fn action_held(&self, action: &str) -> bool {
        self.input_map.bindings.get(action)
            .map_or(false, |bs| bs.iter().any(|b| b.held(&self.state)))
    }

    pub fn action_released(&self, action: &str) -> bool {
        self.input_map.bindings.get(action)
            .map_or(false, |bs| bs.iter().any(|b| b.released(&self.state)))
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
        let frame_gen = Arc::clone(&self.frame_gen);
        app.get_module_mut::<WinitModule>().add_event_hook(move |event| {
            if matches!(event, Event::AboutToWait) {
                frame_gen.fetch_add(1, Ordering::Relaxed);
                return;
            }

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
                    WindowEvent::Focused(false) => Some(RawInputEvent::FocusLost),
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
