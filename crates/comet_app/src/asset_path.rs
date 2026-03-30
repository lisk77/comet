use std::path::{Path, PathBuf};

const RESOURCE_SCHEME: &str = "res://";

pub fn asset_root() -> PathBuf {
    if let Ok(root) = std::env::var("COMET_ASSET_ROOT") {
        return PathBuf::from(root);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let mut ancestors = cwd.as_path().ancestors();
    while let Some(path) = ancestors.next() {
        let candidate = path.join("res");
        if candidate.is_dir() {
            return candidate;
        }
    }

    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let candidate = PathBuf::from(out_dir).join("res");
        if candidate.exists() {
            return candidate;
        }
    }

    cwd.join("res")
}

pub fn file_extension<'a>(resolved: &'a Path, original_path: &str) -> anyhow::Result<&'a str> {
    resolved
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| anyhow::anyhow!("Path '{}' has no file extension", original_path))
}

/// Resolves a path, expanding the `res://` scheme to the engine's resource root.
///
/// - `res://textures/icon.png` → `<asset_root>/textures/icon.png`
/// - Absolute paths are returned as-is.
/// - Relative paths are returned as-is.
pub fn resolve_asset_path(path: impl AsRef<str>) -> PathBuf {
    let path = path.as_ref();

    if let Some(relative) = path.strip_prefix(RESOURCE_SCHEME) {
        return asset_root().join(relative);
    }

    let path = Path::new(path);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    path.to_path_buf()
}
