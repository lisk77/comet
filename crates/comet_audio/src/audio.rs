use comet_assets::{Asset, AssetProvider, AudioClip};

pub trait Audio: Send {
    fn new() -> Self
    where
        Self: Sized;
    fn set_asset_provider(&mut self, provider: AssetProvider);
    fn play(&mut self, clip: Asset<AudioClip>, looped: bool);
    fn pause(&mut self, clip: Asset<AudioClip>);
    fn stop(&mut self, clip: Asset<AudioClip>);
    fn stop_all(&mut self);
    fn update(&mut self, dt: f32);
    fn is_playing(&self, clip: Asset<AudioClip>) -> bool;
    fn set_volume(&mut self, clip: Asset<AudioClip>, volume: f32);
}
