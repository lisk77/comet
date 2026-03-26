use std::any::Any;
use std::collections::HashMap;
use anyhow::{anyhow, Result};

pub struct LoaderRegistry {
    loaders: HashMap<String, Box<dyn Fn(&[u8], &str) -> Result<Box<dyn Any + Send + Sync>> + Send + Sync>>,
}

impl LoaderRegistry {
    pub fn new() -> Self {
        Self {
            loaders: HashMap::new(),
        }
    }

    pub fn register<T: Any + Send + Sync + 'static>(
        &mut self,
        ext: impl Into<String>,
        loader: impl Fn(&[u8], &str) -> Result<T> + Send + Sync + 'static,
    ) {
        let erased = Box::new(move |bytes: &[u8], path: &str| -> Result<Box<dyn Any + Send + Sync>> {
            let value = loader(bytes, path)?;
            Ok(Box::new(value) as Box<dyn Any + Send + Sync>)
        });
        self.loaders.insert(ext.into(), erased);
    }

    pub fn load_bytes<T: Any + Send + Sync + 'static>(
        &self,
        bytes: &[u8],
        path: &str,
        ext: &str,
    ) -> Result<T> {
        let loader = self.loaders.get(ext)
            .ok_or_else(|| anyhow!("No loader registered for extension '{}'", ext))?;
        let boxed = loader(bytes, path)?;
        boxed
            .downcast::<T>()
            .map(|b| *b)
            .map_err(|_| anyhow!("Loader for '{}' returned wrong type", ext))
    }
}
