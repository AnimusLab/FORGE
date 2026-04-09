use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use crate::api::SharedState;
use crate::auth::oauth2::{OAuth2Config, exchange_code, save_refresh_token};
use crate::config::StorageConfig;

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub error: Option<String>,
}

pub async fn login(State(state): State<SharedState>) -> Response {
    let st = state.read().await;

    let oauth_config = match build_oauth_config(&st) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
                .into_response()
        }
    };

    let url = oauth_config.authorization_url();
    tracing::info!("[AUTH] Redirecting to authorization URL");
    Redirect::temporary(&url).into_response()
}

pub async fn callback(
    State(state): State<SharedState>,
    Query(params): Query<CallbackParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Some(err) = params.error {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("OAuth2 error: {}", err) })),
        );
    }

    let code = match params.code {
        Some(c) => c,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing authorization code" })),
            )
        }
    };

    let st = state.read().await;

    let oauth_config = match build_oauth_config(&st) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
        }
    };

    drop(st);

    tracing::info!("[AUTH] Exchanging authorization code for tokens");

    match exchange_code(&oauth_config, &code).await {
        Ok(tokens) => {
            if let Some(refresh_token) = &tokens.refresh_token {
                match save_refresh_token("forge.toml", refresh_token) {
                    Ok(_) => tracing::info!("[AUTH] Refresh token saved to forge.toml"),
                    Err(e) => tracing::warn!("[AUTH] Failed to save refresh token: {}", e),
                }
            }

            tracing::info!("[AUTH] Authorization complete");
            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "message": "Authorization complete. Refresh token saved to forge.toml.",
                    "access_token": tokens.access_token,
                    "expires_in": tokens.expires_in,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        ),
    }
}

fn build_oauth_config(st: &crate::state::AppState) -> Result<OAuth2Config, String> {
    let config = st.forge_config.storage.as_ref()
        .ok_or("[AUTH] No storage config loaded")?;

    match config {
        StorageConfig::Http(h) => {
            let oauth2 = h.oauth2.as_ref()
                .ok_or("[AUTH] No OAuth2 config in forge.toml")?;

            Ok(OAuth2Config::from_forge_config(
                oauth2,
                &h.auth_url,
                &h.redirect_uri,
                vec![
                    "https://www.googleapis.com/auth/drive.file".to_string(),
                ],
            ))
        }
        StorageConfig::S3(_) => {
            Err("[AUTH] S3 backend does not use OAuth2".to_string())
        }
    }
}
