use crate::asset_path::resolve_asset_path;
use comet_log::error;

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

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}
