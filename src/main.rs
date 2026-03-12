mod api;
mod auth;
mod format;
mod state;

use axum::{middleware, Router};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::info;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter("forge=debug")
        .init();

    if std::env::var("FORGE_API_KEY").is_err() {
        panic!("[FORGE] FORGE_API_KEY is not set. Refusing to start.");
    }

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();

    let state = Arc::new(RwLock::new(AppState::new()));

    let app = Router::new()
        .merge(api::public_routes())
        .merge(
            api::protected_routes(state.clone())
                .layer(middleware::from_fn(auth::apikey::auth_middleware)),
        );

    info!("[FORGE] Engine live → http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}