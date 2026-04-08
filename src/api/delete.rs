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
) -> (StatusCode, Json<Value>) {
    let mut st = state.write().await;

    let wal_entry = WalEntry::new(
        Uuid::new_v4().to_string(),
        WalOp::Delete,
        collection.clone(),
        id.clone(),
        None,
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
        Some(file) => {
            if file.delete(&id) {
                let _ = st.wal.mark_committed(&wal_entry.entry_id);
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
