use async_trait::async_trait;
use tokio::net::TcpStream;
use hyper::{Request, Method};
use hyper_util::rt::TokioIo;
use http_body_util::Full;
use bytes::Bytes;
use crate::storage::StorageBackend;
use crate::config::HttpStorageConfig;
use crate::auth::oauth2::{refresh_access_token, OAuth2Config};

pub struct HttpStorageAdapter {
    config: HttpStorageConfig,
}

impl HttpStorageAdapter {
    pub fn new(config: HttpStorageConfig) -> Self {
        Self { config }
    }

    /// Helper to grab a fresh access token using our scratch-built OAuth2 flow
    async fn get_token(&self) -> Result<String, String> {
        let oauth_config_raw = self.config.oauth2.as_ref().ok_or("[STORAGE] Missing OAuth2 config")?;
        
        // Map the config to the OAuth2Config struct
        let oauth_config = OAuth2Config {
            client_id: oauth_config_raw.client_id.clone(),
            client_secret: oauth_config_raw.client_secret.clone(),
            token_url: oauth_config_raw.token_url.clone(),
            redirect_uri: "".to_string(),
            auth_url: "".to_string(),
            scopes: vec![],
        };

        let refresh_token = oauth_config_raw.refresh_token.as_ref().ok_or("[STORAGE] No refresh token found")?;
        
        let token_response = refresh_access_token(&oauth_config, refresh_token).await?;
        Ok(token_response.access_token)
    }

    /// Manual multipart upload to Google Drive using raw HTTP over TLS  
    async fn upload_to_drive(&self, name: &str, boundary: &str, body_bytes: &[u8], token: &str) -> Result<(), String> {
        let host = "www.googleapis.com";
        let path = "/upload/drive/v3/files?uploadType=multipart";

        // Connect and wrap in TLS (same pattern as oauth2.rs)
        let stream = TcpStream::connect(format!("{}:443", host))
            .await
            .map_err(|e| format!("[STORAGE] TCP connect failed: {}", e))?;

        let tls = crate::auth::oauth2::native_tls_connect(stream, host).await?;
        let io = TokioIo::new(tls);

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
            .await
            .map_err(|e| format!("[STORAGE] Handshake failed: {}", e))?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                tracing::warn!("[STORAGE] Connection error: {}", e);
            }
        });

        let req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header("Host", host)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", format!("multipart/related; boundary={}", boundary))
            .header("Content-Length", body_bytes.len().to_string())
            .body(Full::new(Bytes::copy_from_slice(body_bytes)))
            .map_err(|e| format!("[STORAGE] Request build failed: {}", e))?;

        let res = sender.send_request(req)
            .await
            .map_err(|e| format!("[STORAGE] Request send failed: {}", e))?;

        if res.status().is_success() {
            tracing::info!("[STORAGE] Successfully uploaded {}.forge to Google Drive", name);
            Ok(())
        } else {
            Err(format!("[STORAGE] Upload failed with status: {}", res.status()))
        }
    }
}

#[async_trait]
impl StorageBackend for HttpStorageAdapter {
    async fn upload(&self, name: &str, data: &[u8]) -> Result<(), String> {
        let token = self.get_token().await?;
        
        // Manually construct the multipart/related body
        let boundary = "forge_engine_boundary_123456";
        let mut body_bytes = Vec::new();

        // Part 1: Metadata (JSON)
        let metadata = format!(r#"{{"name": "{}.forge", "description": "FORGE Engine Data"}}"#, name);
        body_bytes.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body_bytes.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
        body_bytes.extend_from_slice(metadata.as_bytes());
        body_bytes.extend_from_slice(b"\r\n");

        // Part 2: Binary Data (.forge file)
        body_bytes.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body_bytes.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body_bytes.extend_from_slice(data);
        body_bytes.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

        self.upload_to_drive(name, boundary, &body_bytes, &token).await
    }

    async fn download(&self, _name: &str) -> Result<Vec<u8>, String> {
        tracing::warn!("[STORAGE] Download not yet implemented");
        Ok(vec![])
    }

    async fn list(&self) -> Result<Vec<String>, String> {
        tracing::warn!("[STORAGE] List not yet implemented");
        Ok(vec![])
    }

    async fn delete(&self, _name: &str) -> Result<(), String> {
        tracing::warn!("[STORAGE] Delete not yet implemented");
        Ok(())
    }
}

