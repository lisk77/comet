use crate::audio::Audio;
use comet_assets::AssetProvider;
use kira::{
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    AudioManager, AudioManagerSettings, Decibels, Tween,
};
use std::{collections::HashMap, io::Cursor, sync::Arc};

pub struct KiraAudio {
    manager: AudioManager,
    sounds: HashMap<String, StaticSoundData>,
    handles: HashMap<String, StaticSoundHandle>,
    pending_plays: Vec<(String, bool)>,
    asset_provider: Option<Arc<AssetProvider>>,
}

impl Audio for KiraAudio {
    fn new() -> Self {
        Self {
            manager: AudioManager::new(AudioManagerSettings::default()).unwrap(),
            sounds: HashMap::new(),
            handles: HashMap::new(),
            pending_plays: Vec::new(),
            asset_provider: None,
        }
    }

    fn set_asset_provider(&mut self, provider: Arc<AssetProvider>) {
        self.asset_provider = Some(provider);
    }

    fn play(&mut self, name: &str, looped: bool) {
        if !self.sounds.contains_key(name) {
            let Some(provider) = &self.asset_provider else { return; };
            let Some(handle) = provider.find_by_stem::<comet_assets::AudioClip>(name) else { return; };
            match provider.load_state(handle) {
                comet_assets::LoadState::Ready => {
                    let bytes = provider.with(handle, |c| c.bytes().to_vec());
                    let Some(bytes) = bytes else { return; };
                    match StaticSoundData::from_cursor(Cursor::new(bytes)) {
                        Ok(sound) => { self.sounds.insert(name.to_string(), sound); }
                        Err(e) => { eprintln!("Failed to decode audio clip '{}': {}", name, e); return; }
                    }
                }
                comet_assets::LoadState::Loading => {
                    self.pending_plays.push((name.to_string(), looped));
                    return;
                }
                comet_assets::LoadState::Failed => return,
            }
        }

        if let Some(sound) = self.sounds.get(name) {
            let mut settings = StaticSoundSettings::default();
            if looped { settings = settings.loop_region(..); }
            if let Ok(handle) = self.manager.play(sound.clone().with_settings(settings)) {
                self.handles.insert(name.to_string(), handle);
            }
        }
    }

    fn pause(&mut self, name: &str) {
        if let Some(handle) = self.handles.get_mut(name) {
            handle.pause(Tween::default());
        }
    }

    fn stop(&mut self, name: &str) {
        if let Some(handle) = self.handles.get_mut(name) {
            handle.stop(Tween::default());
        }
    }

    fn stop_all(&mut self) {
        for handle in self.handles.values_mut() {
            handle.stop(Tween::default());
        }
    }

    fn update(&mut self, _dt: f32) {
        let pending = std::mem::take(&mut self.pending_plays);
        for (name, looped) in pending {
            self.play(&name, looped);
        }
    }

    fn is_playing(&self, name: &str) -> bool {
        self.handles.contains_key(name)
    }

    fn set_volume(&mut self, name: &str, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        let db = if vol == 0.0 {
            Decibels::from(-80.0)
        } else {
            Decibels::from(20.0 * vol.log10())
        };
        if let Some(handle) = self.handles.get_mut(name) {
            handle.set_volume(db, Tween::default());
        }
    }
}
