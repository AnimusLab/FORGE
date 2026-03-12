use std::collections::HashMap;
use crate::format::ForgeFile;

pub struct AppState {
    pub collections: HashMap<String, ForgeFile>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, name: &str) -> &mut ForgeFile {
        self.collections
            .entry(name.to_string())
            .or_insert_with(ForgeFile::new)
    }

    pub fn get(&self, name: &str) -> Option<&ForgeFile> {
        self.collections.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut ForgeFile> {
        self.collections.get_mut(name)
    }

    pub fn collection_names(&self) -> Vec<String> {
        self.collections.keys().cloned().collect()
    }
}