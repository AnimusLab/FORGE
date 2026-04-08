use async_trait::async_trait;
use super::StorageBackend;
use crate::config::HttpStorageConfig;

pub struct HttpStorageAdapter {
    pub config: HttpStorageConfig,
}

impl HttpStorageAdapter {
    pub fn new(config: HttpStorageConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl StorageBackend for HttpStorageAdapter {
    async fn upload(&self, _name: &str, _data: &[u8]) -> Result<(), String> {
        // Sprint 5 Step 4 — hyper HTTP upload
        Err("[HTTP] Upload not yet implemented".to_string())
    }

    async fn download(&self, _name: &str) -> Result<Vec<u8>, String> {
        // Sprint 5 Step 4 — hyper HTTP download
        Err("[HTTP] Download not yet implemented".to_string())
    }

    async fn list(&self) -> Result<Vec<String>, String> {
        // Sprint 5 Step 4 — hyper HTTP list
        Err("[HTTP] List not yet implemented".to_string())
    }

    async fn delete(&self, _name: &str) -> Result<(), String> {
        // Sprint 5 Step 4 — hyper HTTP delete
        Err("[HTTP] Delete not yet implemented".to_string())
    }
}
