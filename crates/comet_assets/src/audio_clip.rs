use crate::AssetSettings;
use comet_app::resolve_asset_path;
use comet_log::error;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioSettings;

impl AudioSettings {
    pub fn new() -> Self {
        Self
    }
}

impl AssetSettings for AudioSettings {
    type Asset = AudioClip;

    fn load(&self, bytes: &[u8], _path: &str) -> anyhow::Result<AudioClip> {
        Ok(AudioClip::from_bytes(bytes.to_vec()))
    }
}

pub struct AudioClip {
    bytes: Vec<u8>,
}

impl AudioClip {
    pub fn new(path: &str) -> Self {
        let bytes = match std::fs::read(resolve_asset_path(path)) {
            Ok(b) => b,
            Err(e) => {
                error!("Failed to read audio file '{}': {}", path, e);
                Vec::new()
            }
        };
        Self { bytes }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}
