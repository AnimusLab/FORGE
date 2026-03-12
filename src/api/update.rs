use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::api::SharedState;

pub async fn handle(
    State(state): State<SharedState>,
    Path((collection, id)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if !body.is_object() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Body must be a JSON object" })),
        );
    }

    let obj = body.as_object().unwrap();
    let mut st = state.write().await;

    match st.get_mut(&collection) {
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Collection not found" })),
        ),
        Some(file) => match file.update(&id, obj) {
            Ok(true) => {
                tracing::debug!("UPDATE → collection='{}' id='{}'", collection, id);
                (
                    StatusCode::OK,
                    Json(json!({ "status": "ok", "collection": collection, "id": id })),
                )
            }
            Ok(false) => (
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