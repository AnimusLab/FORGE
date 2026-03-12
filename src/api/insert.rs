use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::api::SharedState;

pub async fn handle(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if !body.is_object() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Body must be a JSON object" })),
        );
    }

    let id = Uuid::new_v4().to_string();
    let obj = body.as_object().unwrap();

    let mut st = state.write().await;
    let file = st.get_or_create(&collection);

    match file.insert(id.clone(), obj) {
        Ok(_) => {
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