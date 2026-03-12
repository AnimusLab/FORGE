use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

pub async fn handle() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "engine": "FORGE",
            "version": "v1"
        })),
    )
}