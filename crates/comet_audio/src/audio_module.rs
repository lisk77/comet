use comet_macros::module;
use comet_app::{App, Module};
use crate::kira::KiraAudio;
use crate::audio::Audio;

pub struct AudioModule {
    audio: KiraAudio,
}

impl AudioModule {
    pub fn new() -> Self {
        Self {
            audio: KiraAudio::new(),
        }
    }
}

impl Module for AudioModule {
    fn dependencies(app: &mut App) where Self: Sized {
        if !app.has_module::<comet_assets::AssetModule>() {
            app.add_module(comet_assets::AssetModule::new());
        }
    }

    fn build(&mut self, app: &mut App) {
        self.audio.set_asset_provider(app.context::<comet_assets::AssetProvider>().clone());
        app.add_tick_system(|app, dt| {
            app.get_module_mut::<AudioModule>().audio.update(dt);
        });
    }
}

#[module]
impl AudioModule {
    pub fn play_audio(&mut self, name: &str, looped: bool) {
        self.audio.play(name, looped);
    }

    pub fn pause_audio(&mut self, name: &str) {
        self.audio.pause(name);
    }

    pub fn stop_audio(&mut self, name: &str) {
        self.audio.stop(name);
    }

    pub fn stop_all_audio(&mut self) {
        self.audio.stop_all();
    }

    pub fn is_playing(&self, name: &str) -> bool {
        self.audio.is_playing(name)
    }

    pub fn set_volume(&mut self, name: &str, volume: f32) {
        self.audio.set_volume(name, volume);
    }
}
