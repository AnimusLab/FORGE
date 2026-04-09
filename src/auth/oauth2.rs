use hyper::{Method, Request, body::Bytes};
use hyper_util::rt::TokioIo;
use http_body_util::{BodyExt, Full};
use tokio::net::TcpStream;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub token_type: String,
}

#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuth2Config {
    pub fn from_forge_config(config: &crate::config::OAuthConfig, auth_url: &str, redirect_uri: &str, scopes: Vec<String>) -> Self {
        Self {
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone(),
            auth_url: auth_url.to_string(),
            token_url: config.token_url.clone(),
            redirect_uri: redirect_uri.to_string(),
            scopes,
        }
    }

    // Build the URL the user visits to authorize FORGE
    pub fn authorization_url(&self) -> String {
        let scopes = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent",
            self.auth_url,
            url_encode(&self.client_id),
            url_encode(&self.redirect_uri),
            url_encode(&scopes),
        )
    }
}

// Exchange authorization code for tokens
pub async fn exchange_code(config: &OAuth2Config, code: &str) -> Result<TokenResponse, String> {
    let body = format!(
        "client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri={}",
        url_encode(&config.client_id),
        url_encode(&config.client_secret),
        url_encode(code),
        url_encode(&config.redirect_uri),
    );

    post_to_token_endpoint(&config.token_url, &body).await
}

// Use refresh token to get a new access token
pub async fn refresh_access_token(config: &OAuth2Config, refresh_token: &str) -> Result<TokenResponse, String> {
    let body = format!(
        "client_id={}&client_secret={}&refresh_token={}&grant_type=refresh_token",
        url_encode(&config.client_id),
        url_encode(&config.client_secret),
        url_encode(refresh_token),
    );

    post_to_token_endpoint(&config.token_url, &body).await
}

// Save refresh token back to forge.toml automatically
pub fn save_refresh_token(path: &str, refresh_token: &str) -> Result<(), String> {
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("[OAUTH2] Failed to read {}: {}", path, e))?;

    let updated = if contents.contains("refresh_token = \"\"") {
        contents.replace(
            "refresh_token = \"\"",
            &format!("refresh_token = \"{}\"", refresh_token),
        )
    } else {
        // Replace existing refresh token
        let mut lines: Vec<String> = contents.lines().map(|l| l.to_string()).collect();
        for line in &mut lines {
            if line.trim_start().starts_with("refresh_token") {
                *line = format!("refresh_token = \"{}\"", refresh_token);
            }
        }
        lines.join("\n")
    };

    fs::write(path, updated)
        .map_err(|e| format!("[OAUTH2] Failed to write {}: {}", path, e))?;

    tracing::info!("[OAUTH2] Refresh token saved to {}", path);
    Ok(())
}

// Raw HTTP POST to a token endpoint using hyper — no reqwest
async fn post_to_token_endpoint(token_url: &str, body: &str) -> Result<TokenResponse, String> {
    let (host, path) = parse_url(token_url)?;

    let stream = TcpStream::connect(format!("{}:443", host))
        .await
        .map_err(|e| format!("[OAUTH2] TCP connect failed: {}", e))?;

    // Wrap in TLS
    let tls = native_tls_connect(stream, &host).await?;
    let io = TokioIo::new(tls);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| format!("[OAUTH2] Handshake failed: {}", e))?;

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            tracing::warn!("[OAUTH2] Connection error: {}", e);
        }
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header("Host", &host)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", body.len().to_string())
        .body(Full::new(Bytes::from(body.to_string())))
        .map_err(|e| format!("[OAUTH2] Request build failed: {}", e))?;

    let res = sender.send_request(req)
        .await
        .map_err(|e| format!("[OAUTH2] Request failed: {}", e))?;

    let status = res.status();
    let bytes = res.into_body().collect().await
        .map_err(|e| format!("[OAUTH2] Body read failed: {}", e))?
        .to_bytes();

    let text = String::from_utf8_lossy(&bytes).to_string();

    if !status.is_success() {
        return Err(format!("[OAUTH2] Token endpoint returned {}: {}", status, text));
    }

    serde_json::from_str::<TokenResponse>(&text)
        .map_err(|e| format!("[OAUTH2] Failed to parse token response: {} — body: {}", e, text))
}

// Minimal URL percent-encoder — no urlencoding crate
fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

// Parse https://host/path into (host, path)
fn parse_url(url: &str) -> Result<(String, String), String> {
    let without_scheme = url
        .strip_prefix("https://")
        .ok_or_else(|| format!("[OAUTH2] URL must start with https://: {}", url))?;

    let slash_pos = without_scheme.find('/').unwrap_or(without_scheme.len());
    let host = without_scheme[..slash_pos].to_string();
    let path = if slash_pos < without_scheme.len() {
        without_scheme[slash_pos..].to_string()
    } else {
        "/".to_string()
    };

    Ok((host, path))
}

// Native TLS handshake over TCP stream
pub async fn native_tls_connect(
    stream: TcpStream,
    host: &str,
) -> Result<tokio_native_tls::TlsStream<TcpStream>, String> {
    let connector = tokio_native_tls::TlsConnector::from(
        native_tls::TlsConnector::new()
            .map_err(|e| format!("[OAUTH2] TLS connector failed: {}", e))?,
    );
    connector
        .connect(host, stream)
        .await
        .map_err(|e| format!("[OAUTH2] TLS handshake failed: {}", e))
}
