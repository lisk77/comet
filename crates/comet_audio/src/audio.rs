use crate::{Playback, PlaybackState};
use comet_assets::{Asset, AssetProvider, AudioClip};
use comet_ecs::Entity;

pub trait Audio: Send {
    fn new() -> Self
    where
        Self: Sized;

    fn set_asset_provider(&mut self, provider: AssetProvider);

    fn ensure_playing(
        &mut self,
        entity: Entity,
        clip: Asset<AudioClip>,
        playback: Playback,
        volume: f32,
    );

    fn pause(&mut self, entity: Entity);
    fn stop(&mut self, entity: Entity);
    fn stop_all(&mut self);
    fn update(&mut self, dt: f32) -> Vec<(Entity, PlaybackState)>;
    fn active_entities(&self) -> Vec<Entity>;
}
