use crate::audio::Audio;
use crate::kira::KiraAudio;
use comet_app::{App, Module};
use comet_assets::{Asset, AudioClip};
use comet_macros::module;

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
    fn dependencies(app: &mut App)
    where
        Self: Sized,
    {
        if !app.has_module::<comet_assets::AssetModule>() {
            app.add_module(comet_assets::AssetModule::new());
        }
    }

    fn build(&mut self, app: &mut App) {
        self.audio
            .set_asset_provider(app.context::<comet_assets::AssetProvider>().clone());
        app.add_tick_system(|app, dt| {
            app.get_module_mut::<AudioModule>().audio.update(dt);
        });
    }
}

#[module]
impl AudioModule {
    pub fn play_audio(&mut self, clip: Asset<AudioClip>, looped: bool) {
        self.audio.play(clip, looped);
    }

    pub fn pause_audio(&mut self, clip: Asset<AudioClip>) {
        self.audio.pause(clip);
    }

    pub fn stop_audio(&mut self, clip: Asset<AudioClip>) {
        self.audio.stop(clip);
    }

    pub fn stop_all_audio(&mut self) {
        self.audio.stop_all();
    }

    pub fn is_playing(&self, clip: Asset<AudioClip>) -> bool {
        self.audio.is_playing(clip)
    }

    pub fn set_volume(&mut self, clip: Asset<AudioClip>, volume: f32) {
        self.audio.set_volume(clip, volume);
    }
}
