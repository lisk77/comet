use crate::audio::Audio;
use comet_assets::{Asset, AssetProvider, AudioClip};
use kira::{
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    AudioManager, AudioManagerSettings, Decibels, Tween,
};
use std::{collections::HashMap, io::Cursor};

pub struct KiraAudio {
    manager: AudioManager,
    sounds: HashMap<Asset<AudioClip>, StaticSoundData>,
    handles: HashMap<Asset<AudioClip>, StaticSoundHandle>,
    pending_plays: Vec<(Asset<AudioClip>, bool)>,
    asset_provider: Option<AssetProvider>,
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

    fn set_asset_provider(&mut self, provider: AssetProvider) {
        self.asset_provider = Some(provider);
    }

    fn play(&mut self, clip: Asset<AudioClip>, looped: bool) {
        if !self.sounds.contains_key(&clip) {
            let Some(provider) = &self.asset_provider else {
                return;
            };
            match provider.load_state(clip) {
                comet_assets::LoadState::Ready => {
                    let bytes = provider.with(clip, |c| c.bytes().to_vec());
                    let Some(bytes) = bytes else {
                        return;
                    };
                    match StaticSoundData::from_cursor(Cursor::new(bytes)) {
                        Ok(sound) => {
                            self.sounds.insert(clip, sound);
                        }
                        Err(e) => {
                            eprintln!("Failed to decode audio clip {:?}: {}", clip, e);
                            return;
                        }
                    }
                }
                comet_assets::LoadState::Loading => {
                    self.pending_plays.push((clip, looped));
                    return;
                }
                comet_assets::LoadState::Failed => return,
            }
        }

        if let Some(sound) = self.sounds.get(&clip) {
            let mut settings = StaticSoundSettings::default();
            if looped {
                settings = settings.loop_region(..);
            }
            if let Ok(handle) = self.manager.play(sound.clone().with_settings(settings)) {
                self.handles.insert(clip, handle);
            }
        }
    }

    fn pause(&mut self, clip: Asset<AudioClip>) {
        if let Some(handle) = self.handles.get_mut(&clip) {
            handle.pause(Tween::default());
        }
    }

    fn stop(&mut self, clip: Asset<AudioClip>) {
        if let Some(handle) = self.handles.get_mut(&clip) {
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
        for (clip, looped) in pending {
            self.play(clip, looped);
        }
    }

    fn is_playing(&self, clip: Asset<AudioClip>) -> bool {
        self.handles.contains_key(&clip)
    }

    fn set_volume(&mut self, clip: Asset<AudioClip>, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        let db = if vol == 0.0 {
            Decibels::from(-80.0)
        } else {
            Decibels::from(20.0 * vol.log10())
        };
        if let Some(handle) = self.handles.get_mut(&clip) {
            handle.set_volume(db, Tween::default());
        }
    }
}
