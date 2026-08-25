use comet_assets::{Asset, AudioClip};
use comet_ecs::{Component, RequiredComponents};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Playback {
    #[default]
    Once,
    Loop,
    Repeat(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Volume(f32);

impl Default for Volume {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component)]
pub struct PlaybackSettings {
    playback: Playback,
    volume: Volume,
}

impl PlaybackSettings {
    pub const ONCE: Self = Self {
        playback: Playback::Once,
        volume: Volume(1.0),
    };

    pub const LOOP: Self = Self {
        playback: Playback::Loop,
        volume: Volume(1.0),
    };

    pub const fn repeat(total_plays: usize) -> Self {
        Self {
            playback: Playback::Repeat(total_plays),
            volume: Volume(1.0),
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.set_volume(volume);
        self
    }

    pub fn playback(&self) -> Playback {
        self.playback
    }

    pub fn set_playback(&mut self, playback: Playback) {
        self.playback = playback;
    }

    pub fn volume(&self) -> f32 {
        self.volume.0
    }

    pub fn set_volume(&mut self, volume: f32) {
        assert!(
            volume.is_finite() && (0.0..=1.0).contains(&volume),
            "audio volume must be finite and between zero and one"
        );
        self.volume = Volume(volume);
    }
}

#[derive(Component, Copy, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Finished,
}

impl PlaybackState {
    pub fn play(&mut self) {
        *self = Self::Playing;
    }

    pub fn pause(&mut self) {
        *self = Self::Paused;
    }

    pub fn stop(&mut self) {
        *self = Self::Stopped;
    }

    pub(crate) fn finish(&mut self) {
        *self = Self::Finished;
    }
}

#[derive(Component)]
#[require(PlaybackSettings, PlaybackState)]
pub struct AudioSource {
    clip: Asset<AudioClip>,
}

impl AudioSource {
    pub fn new(clip: Asset<AudioClip>) -> Self {
        Self { clip }
    }

    pub fn clip(&self) -> Asset<AudioClip> {
        self.clip
    }

    pub fn set_clip(&mut self, clip: Asset<AudioClip>) {
        self.clip = clip;
    }
}
