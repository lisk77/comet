use crate::asset_path::resolve_asset_path;

pub struct AudioClip {
    bytes: Vec<u8>,
}

impl AudioClip {
    pub fn new(path: &str) -> Self {
        let bytes = std::fs::read(resolve_asset_path(path))
            .unwrap_or_else(|e| panic!("Failed to read audio file '{}': {}", path, e));
        Self { bytes }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
