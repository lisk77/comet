use crate::audio::Audio;
use kira::{
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    AudioManager, AudioManagerSettings, Decibels, Tween,
};
use std::{collections::HashMap, path::Path};

pub struct KiraAudio {
    manager: AudioManager,
    sounds: HashMap<String, StaticSoundData>,
    handles: HashMap<String, StaticSoundHandle>,
}

impl KiraAudio {
    fn load_sound(path: &Path) -> Option<StaticSoundData> {
        StaticSoundData::from_file(path).ok()
    }
}

impl Audio for KiraAudio {
    fn new() -> Self {
        Self {
            manager: AudioManager::new(AudioManagerSettings::default()).unwrap(),
            sounds: HashMap::new(),
            handles: HashMap::new(),
        }
    }

    fn load(&mut self, name: &str, path: &str) {
        if let Some(sound) = Self::load_sound(Path::new(path)) {
            self.sounds.insert(name.to_string(), sound);
        }
    }

    fn play(&mut self, name: &str, looped: bool) {
        if let Some(sound) = self.sounds.get(name) {
            let mut settings = StaticSoundSettings::default();

            if looped {
                settings = settings.loop_region(..);
            }

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

    // KiraAudio needs no updating function, it just exists to make the trait happy
    fn update(&mut self, _dt: f32) {}

    fn is_playing(&self, name: &str) -> bool {
        self.handles.contains_key(name)
    }

    fn set_volume(&mut self, name: &str, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        let db = if vol == 0.0 {
            Decibels::from(-80.0) // effectively silent
        } else {
            Decibels::from(20.0 * vol.log10())
        };

        if let Some(handle) = self.handles.get_mut(name) {
            handle.set_volume(db, Tween::default());
        }
    }
}
