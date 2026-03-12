pub mod delete;
pub mod health;
pub mod insert;
pub mod query;
pub mod query_one;
pub mod update;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::state::AppState;

pub type SharedState = Arc<RwLock<AppState>>;

pub fn public_routes() -> Router {
    Router::new()
        .route("/v1/health", get(health::handle))
}

pub fn protected_routes(state: SharedState) -> Router {
    Router::new()
        .route(
            "/v1/data/:collection",
            post(insert::handle).get(query::handle),
        )
        .route(
            "/v1/data/:collection/:id",
            get(query_one::handle)
                .patch(update::handle)
                .delete(delete::handle),
        )
        .route("/v1/collections", get(query::list_collections))
        .with_state(state)
}