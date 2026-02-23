mod api;
mod config;
mod dto;
mod error;
mod infrastucture;
pub mod schema;
mod service;

pub use crate::api::AppState;
use crate::config::AppConfig;
use crate::infrastucture::database::pool::create_pool;

async fn server_start<'a>(addr: &str, state: AppState) {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let router = api::routes::create_router(state);
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}

pub async fn bootstrap() {
    let config = AppConfig::new();
    let db_pool = create_pool(&config.db_config.url)
        .await
        .expect("Failed to connect to the database");
    let app_state = AppState::new(config, db_pool);
    server_start(
        &format!("0.0.0.0:{}", &app_state.config.app_port),
        app_state,
    )
    .await;
}
