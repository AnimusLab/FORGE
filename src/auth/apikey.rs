use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub async fn auth_middleware(req: Request, next: Next) -> Response {
    let forge_key = std::env::var("FORGE_API_KEY").unwrap_or_default();

    let provided = req
        .headers()
        .get("X-Forge-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided.is_empty() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Missing X-Forge-Key header" })),
        )
            .into_response();
    }

    if provided != forge_key {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid API key" })),
        )
            .into_response();
    }

    next.run(req).await
}