use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::api::SharedState;

pub async fn handle(
    State(state): State<SharedState>,
    Path((collection, id)): Path<(String, String)>,
) -> (StatusCode, Json<Value>) {
    let st = state.read().await;

    match st.get(&collection) {
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Collection not found", "collection": collection })),
        ),
        Some(file) => match file.get_one(&id) {
            Ok(Some(record)) => {
                tracing::debug!("QUERY ONE → collection='{}' id='{}'", collection, id);
                (StatusCode::OK, Json(record))
            }
            Ok(None) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Record not found", "id": id })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            ),
        },
    }
}