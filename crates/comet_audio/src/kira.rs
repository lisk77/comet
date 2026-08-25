use crate::{audio::Audio, Playback, PlaybackState};
use comet_assets::{Asset, AssetProvider, AudioClip, LoadState};
use comet_ecs::Entity;
use kira::{
    sound::{
        static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
        PlaybackState as KiraPlaybackState,
    },
    AudioManager, AudioManagerSettings, Decibels, Tween,
};
use std::{collections::HashMap, io::Cursor};

struct Voice {
    clip: Asset<AudioClip>,
    playback: Playback,
    volume: f32,
    remaining_plays: usize,
    handle: Option<StaticSoundHandle>,
}

enum StartOutcome {
    Started,
    Loading,
    Failed,
}

pub struct KiraAudio {
    manager: AudioManager,
    sounds: HashMap<Asset<AudioClip>, StaticSoundData>,
    voices: HashMap<Entity, Voice>,
    failed: Vec<Entity>,
    asset_provider: Option<AssetProvider>,
}

impl KiraAudio {
    fn start_voice(&mut self, entity: Entity) -> StartOutcome {
        let Some(voice) = self.voices.get(&entity) else {
            return StartOutcome::Failed;
        };
        let clip = voice.clip;

        if !self.sounds.contains_key(&clip) {
            let Some(provider) = &self.asset_provider else {
                return StartOutcome::Failed;
            };
            match provider.load_state(clip) {
                LoadState::Ready => {
                    let Some(bytes) = provider.with(clip, |clip| clip.bytes().to_vec()) else {
                        return StartOutcome::Failed;
                    };
                    match StaticSoundData::from_cursor(Cursor::new(bytes)) {
                        Ok(sound) => {
                            self.sounds.insert(clip, sound);
                        }
                        Err(error) => {
                            eprintln!("Failed to decode audio clip {:?}: {}", clip, error);
                            return StartOutcome::Failed;
                        }
                    }
                }
                LoadState::Loading => return StartOutcome::Loading,
                LoadState::Failed => return StartOutcome::Failed,
            }
        }

        let Some(voice) = self.voices.get(&entity) else {
            return StartOutcome::Failed;
        };
        let mut settings = StaticSoundSettings::default().volume(volume_to_decibels(voice.volume));
        if voice.playback == Playback::Loop {
            settings = settings.loop_region(..);
        }
        let Some(sound) = self.sounds.get(&clip) else {
            return StartOutcome::Failed;
        };
        match self.manager.play(sound.clone().with_settings(settings)) {
            Ok(handle) => {
                if let Some(voice) = self.voices.get_mut(&entity) {
                    voice.handle = Some(handle);
                }
                StartOutcome::Started
            }
            Err(_) => StartOutcome::Failed,
        }
    }

    fn replace_voice(
        &mut self,
        entity: Entity,
        clip: Asset<AudioClip>,
        playback: Playback,
        volume: f32,
    ) {
        self.stop(entity);
        let remaining_plays = match playback {
            Playback::Once | Playback::Loop => 0,
            Playback::Repeat(total) => total.saturating_sub(1),
        };
        self.voices.insert(
            entity,
            Voice {
                clip,
                playback,
                volume,
                remaining_plays,
                handle: None,
            },
        );
        if matches!(self.start_voice(entity), StartOutcome::Failed) {
            self.voices.remove(&entity);
            self.failed.push(entity);
        }
    }
}

impl Audio for KiraAudio {
    fn new() -> Self {
        Self {
            manager: AudioManager::new(AudioManagerSettings::default()).unwrap(),
            sounds: HashMap::new(),
            voices: HashMap::new(),
            failed: Vec::new(),
            asset_provider: None,
        }
    }

    fn set_asset_provider(&mut self, provider: AssetProvider) {
        self.asset_provider = Some(provider);
    }

    fn ensure_playing(
        &mut self,
        entity: Entity,
        clip: Asset<AudioClip>,
        playback: Playback,
        volume: f32,
    ) {
        let replace = self
            .voices
            .get(&entity)
            .is_none_or(|voice| voice.clip != clip || voice.playback != playback);
        if replace {
            self.replace_voice(entity, clip, playback, volume);
            return;
        }

        let Some(voice) = self.voices.get_mut(&entity) else {
            return;
        };
        if voice.volume != volume {
            voice.volume = volume;
            if let Some(handle) = &mut voice.handle {
                handle.set_volume(volume_to_decibels(volume), Tween::default());
            }
        }
        if let Some(handle) = &mut voice.handle {
            if handle.state() == KiraPlaybackState::Paused {
                handle.resume(Tween::default());
            }
        }
    }

    fn pause(&mut self, entity: Entity) {
        if let Some(handle) = self
            .voices
            .get_mut(&entity)
            .and_then(|voice| voice.handle.as_mut())
        {
            handle.pause(Tween::default());
        }
    }

    fn stop(&mut self, entity: Entity) {
        self.failed.retain(|failed| *failed != entity);
        if let Some(mut voice) = self.voices.remove(&entity) {
            if let Some(handle) = &mut voice.handle {
                handle.stop(Tween::default());
            }
        }
    }

    fn stop_all(&mut self) {
        for voice in self.voices.values_mut() {
            if let Some(handle) = &mut voice.handle {
                handle.stop(Tween::default());
            }
        }
        self.voices.clear();
        self.failed.clear();
    }

    fn update(&mut self, _dt: f32) -> Vec<(Entity, PlaybackState)> {
        let mut restart = Vec::new();
        let mut finished = std::mem::take(&mut self.failed);

        for (entity, voice) in &mut self.voices {
            match voice.handle.as_ref().map(StaticSoundHandle::state) {
                None => restart.push(*entity),
                Some(KiraPlaybackState::Stopped) if voice.remaining_plays > 0 => {
                    voice.remaining_plays -= 1;
                    voice.handle = None;
                    restart.push(*entity);
                }
                Some(KiraPlaybackState::Stopped) => finished.push(*entity),
                _ => {}
            }
        }

        for entity in restart {
            if matches!(self.start_voice(entity), StartOutcome::Failed) {
                finished.push(entity);
            }
        }
        finished.sort_unstable();
        finished.dedup();
        for entity in &finished {
            self.voices.remove(entity);
        }
        finished
            .into_iter()
            .map(|entity| (entity, PlaybackState::Finished))
            .collect()
    }

    fn active_entities(&self) -> Vec<Entity> {
        self.voices.keys().copied().collect()
    }
}

fn volume_to_decibels(volume: f32) -> Decibels {
    if volume == 0.0 {
        Decibels::from(-80.0)
    } else {
        Decibels::from(20.0 * volume.log10())
    }
}
