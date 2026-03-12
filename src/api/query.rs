use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::api::SharedState;

pub async fn handle(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
) -> (StatusCode, Json<Value>) {
    let st = state.read().await;

    match st.get(&collection) {
        None => (
            StatusCode::OK,
            Json(json!({ "collection": collection, "records": [] })),
        ),
        Some(file) => match file.get_all() {
            Ok(records) => {
                tracing::debug!("QUERY ALL → collection='{}' count={}", collection, records.len());
                (
                    StatusCode::OK,
                    Json(json!({ "collection": collection, "records": records })),
                )
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            ),
        },
    }
}

pub async fn list_collections(
    State(state): State<SharedState>,
) -> (StatusCode, Json<Value>) {
    let st = state.read().await;
    let names = st.collection_names();
    (
        StatusCode::OK,
        Json(json!({ "collections": names })),
    )
}