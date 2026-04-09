use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct ForgeConfig {
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum StorageConfig {
    #[serde(rename = "http")]
    Http(HttpStorageConfig),

    #[serde(rename = "s3")]
    S3(S3StorageConfig),
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpStorageConfig {
    pub base_url: String,
    pub auth_type: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub oauth2: Option<OAuthConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: Option<String>,
    pub token_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3StorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

pub fn load(path: &str) -> Result<ForgeConfig, String> {
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("[CONFIG] Failed to read {}: {}", path, e))?;

    toml::from_str(&contents)
        .map_err(|e| format!("[CONFIG] Failed to parse forge.toml: {}", e))
}
