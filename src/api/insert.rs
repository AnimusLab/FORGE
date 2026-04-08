use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::api::SharedState;
use crate::engine::wal::{WalEntry, WalOp};

pub async fn handle(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(collection): Path<String>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if !body.is_object() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Body must be a JSON object" })),
        );
    }

    let request_id = headers
        .get("X-Forge-Request-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let mut st = state.write().await;

    if !st.idempotency.check_and_insert(&request_id) {
        tracing::debug!("DUPLICATE request_id='{}' → ignored", request_id);
        return (
            StatusCode::OK,
            Json(json!({
                "status": "duplicate",
                "request_id": request_id,
                "message": "Request already processed"
            })),
        );
    }

    let id = Uuid::new_v4().to_string();
    let obj = body.as_object().unwrap();

    let wal_entry = WalEntry::new(
        Uuid::new_v4().to_string(),
        WalOp::Insert,
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

    let file = st.get_or_create(&collection);
    match file.insert(id.clone(), obj) {
        Ok(_) => {
            let _ = st.wal.mark_committed(&wal_entry.entry_id);
            tracing::debug!("INSERT → collection='{}' id='{}'", collection, id);
            (
                StatusCode::CREATED,
                Json(json!({
                    "status": "ok",
                    "collection": collection,
                    "id": id
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        ),
    }
}
