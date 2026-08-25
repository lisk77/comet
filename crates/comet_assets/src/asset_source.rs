use crate::Asset;
use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AssetPath(Arc<str>);

impl AssetPath {
    pub fn new(path: impl AsRef<str>) -> Self {
        Self(Arc::from(path.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AssetPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("AssetPath")
            .field(&self.as_str())
            .finish()
    }
}

impl fmt::Display for AssetPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for AssetPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for AssetPath {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl From<String> for AssetPath {
    fn from(path: String) -> Self {
        Self(Arc::from(path))
    }
}

impl From<Arc<str>> for AssetPath {
    fn from(path: Arc<str>) -> Self {
        Self(path)
    }
}

pub enum AssetSource<T> {
    Path(AssetPath),
    Handle(Asset<T>),
}

impl<T> AssetSource<T> {
    pub fn path(&self) -> Option<&AssetPath> {
        match self {
            Self::Path(path) => Some(path),
            Self::Handle(_) => None,
        }
    }

    pub fn handle(&self) -> Option<Asset<T>> {
        match self {
            Self::Path(_) => None,
            Self::Handle(handle) => Some(*handle),
        }
    }
}

impl<T> Clone for AssetSource<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Path(path) => Self::Path(path.clone()),
            Self::Handle(handle) => Self::Handle(*handle),
        }
    }
}

impl<T> fmt::Debug for AssetSource<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => formatter.debug_tuple("Path").field(path).finish(),
            Self::Handle(handle) => formatter.debug_tuple("Handle").field(handle).finish(),
        }
    }
}

impl<T> PartialEq for AssetSource<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Path(left), Self::Path(right)) => left == right,
            (Self::Handle(left), Self::Handle(right)) => left == right,
            _ => false,
        }
    }
}

impl<T> Eq for AssetSource<T> {}

impl<T> Hash for AssetSource<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Path(path) => {
                0u8.hash(state);
                path.hash(state);
            }
            Self::Handle(handle) => {
                1u8.hash(state);
                handle.hash(state);
            }
        }
    }
}

impl<T> From<Asset<T>> for AssetSource<T> {
    fn from(handle: Asset<T>) -> Self {
        Self::Handle(handle)
    }
}

impl<T> From<AssetPath> for AssetSource<T> {
    fn from(path: AssetPath) -> Self {
        Self::Path(path)
    }
}

impl<T> From<&str> for AssetSource<T> {
    fn from(path: &str) -> Self {
        Self::Path(path.into())
    }
}

impl<T> From<String> for AssetSource<T> {
    fn from(path: String) -> Self {
        Self::Path(path.into())
    }
}
