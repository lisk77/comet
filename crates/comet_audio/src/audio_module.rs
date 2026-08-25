use crate::{audio::Audio, AudioSource, KiraAudio, Playback, PlaybackSettings, PlaybackState};
use comet_app::{App, Module};
use comet_ecs::{EcsModule, EcsModuleExt, Entity, Scene};

pub struct AudioModule {
    audio: KiraAudio,
}

impl AudioModule {
    pub fn new() -> Self {
        Self {
            audio: KiraAudio::new(),
        }
    }

    fn update(&mut self, scene: &mut Scene, dt: f32) {
        for (entity, state) in self.audio.update(dt) {
            if state == PlaybackState::Finished {
                if let Some(playback_state) = scene.get_component_mut::<PlaybackState>(entity) {
                    playback_state.finish();
                }
            }
        }

        for entity in self.audio.active_entities() {
            if scene.get_component::<AudioSource>(entity).is_none() {
                self.audio.stop(entity);
            }
        }

        for (entity, source, settings, state) in scene
            .query_mut::<(Entity, &AudioSource, &PlaybackSettings, &mut PlaybackState), ()>()
        {
            if settings.playback() == Playback::Repeat(0) {
                self.audio.stop(entity);
                state.finish();
                continue;
            }

            match *state {
                PlaybackState::Playing => self.audio.ensure_playing(
                    entity,
                    source.clip(),
                    settings.playback(),
                    settings.volume(),
                ),
                PlaybackState::Paused => self.audio.pause(entity),
                PlaybackState::Stopped | PlaybackState::Finished => self.audio.stop(entity),
            }
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
        if !app.has_module::<EcsModule>() {
            app.add_module(EcsModule::new());
        }
    }

    fn build(&mut self, app: &mut App) {
        self.audio
            .set_asset_provider(app.context::<comet_assets::AssetProvider>().clone());
        app.add_tick_system(|app, dt| {
            let mut audio = app.take_module::<AudioModule>().unwrap();
            let mut ecs = app.take_module::<EcsModule>().unwrap();
            audio.update(&mut ecs.scene, dt);
            app.reinsert_module(ecs);
            app.reinsert_module(audio);
        });
    }
}

pub trait AudioModuleExt {
    fn pause_audio(&mut self, entity: Entity);
    fn resume_audio(&mut self, entity: Entity);
    fn stop_audio(&mut self, entity: Entity);
    fn stop_all_audio(&mut self);
    fn is_audio_playing(&self, entity: Entity) -> bool;
    fn audio_state(&self, entity: Entity) -> Option<PlaybackState>;
    fn set_audio_volume(&mut self, entity: Entity, volume: f32);
}

impl AudioModuleExt for App {
    fn pause_audio(&mut self, entity: Entity) {
        if let Some(state) = self.get_component_mut::<PlaybackState>(entity) {
            state.pause();
        }
    }

    fn resume_audio(&mut self, entity: Entity) {
        if let Some(state) = self.get_component_mut::<PlaybackState>(entity) {
            state.play();
        }
    }

    fn stop_audio(&mut self, entity: Entity) {
        if let Some(state) = self.get_component_mut::<PlaybackState>(entity) {
            state.stop();
        }
    }

    fn stop_all_audio(&mut self) {
        for state in self.query::<&mut PlaybackState, ()>() {
            state.stop();
        }
        self.get_module_mut::<AudioModule>().audio.stop_all();
    }

    fn is_audio_playing(&self, entity: Entity) -> bool {
        self.audio_state(entity) == Some(PlaybackState::Playing)
    }

    fn audio_state(&self, entity: Entity) -> Option<PlaybackState> {
        self.get_component::<PlaybackState>(entity).copied()
    }

    fn set_audio_volume(&mut self, entity: Entity, volume: f32) {
        if let Some(settings) = self.get_component_mut::<PlaybackSettings>(entity) {
            settings.set_volume(volume);
        }
    }
}
