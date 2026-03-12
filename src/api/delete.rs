use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::api::SharedState;

pub async fn handle(
    State(state): State<SharedState>,
    Path((collection, id)): Path<(String, String)>,
) -> (StatusCode, Json<Value>) {
    let mut st = state.write().await;

    match st.get_mut(&collection) {
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Collection not found" })),
        ),
        Some(file) => {
            if file.delete(&id) {
                tracing::debug!("DELETE → collection='{}' id='{}'", collection, id);
                (
                    StatusCode::OK,
                    Json(json!({ "status": "ok", "collection": collection, "id": id })),
                )
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Record not found", "id": id })),
                )
            }
        }
    }
}