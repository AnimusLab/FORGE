use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::api::SharedState;
use crate::engine::wal::{WalEntry, WalOp};

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

    let wal_entry = WalEntry::new(
        Uuid::new_v4().to_string(),
        WalOp::Update,
        collection.clone(),
        id.clone(),
        Some(body.clone()),
    );

    if let Err(e) = st.wal.append(&wal_entry) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("WAL write failed: {}", e) })),
        );
    }

    match st.get_mut(&collection) {
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Collection not found" })),
        ),
        Some(file) => match file.update(&id, obj) {
            Ok(true) => {
                let _ = st.wal.mark_committed(&wal_entry.entry_id);
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