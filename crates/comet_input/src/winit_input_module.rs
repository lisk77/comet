use std::sync::{Arc, Mutex};
use comet_app::{App, Module};
use comet_macros::module;
use comet_window::WinitModule;
use winit_input_helper::WinitInputHelper;
use crate::keyboard::Key;

pub struct WinitInputModule {
    inner: Arc<Mutex<WinitInputHelper>>,
}

impl WinitInputModule {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(WinitInputHelper::new())) }
    }
}

#[module]
impl WinitInputModule {
    pub fn key_pressed(&self, key: Key) -> bool {
        self.inner.lock().unwrap().key_pressed(key)
    }

    pub fn key_held(&self, key: Key) -> bool {
        self.inner.lock().unwrap().key_held(key)
    }

    pub fn key_released(&self, key: Key) -> bool {
        self.inner.lock().unwrap().key_released(key)
    }
}

impl Module for WinitInputModule {
    fn dependencies(app: &mut App) where Self: Sized {
        if !app.has_module::<WinitModule>() {
            app.add_module(WinitModule::new());
        }
    }

    fn build(&mut self, app: &mut App) {
        let arc = Arc::clone(&self.inner);
        app.get_module_mut::<WinitModule>().add_event_hook(move |event| {
            if let Ok(mut m) = arc.lock() {
                m.update(event);
            }
        });
    }
}
