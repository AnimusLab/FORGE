pub mod http;

use async_trait::async_trait;

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn upload(&self, name: &str, data: &[u8]) -> Result<(), String>;
    async fn download(&self, name: &str) -> Result<Vec<u8>, String>;
    async fn list(&self) -> Result<Vec<String>, String>;
    async fn delete(&self, name: &str) -> Result<(), String>;
}
