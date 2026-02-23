use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use super::{AppState, handlers};
use axum::Router;

pub fn create_router(state: AppState) -> Router {
    let state = Arc::new(state);
    let (router, api) = OpenApiRouter::new()
        .nest("/api/v1/auth", handlers::auth::auth_router(state.clone()))
        .nest(
            "/api/v1/sign-up",
            handlers::sign_up::sign_up_router(state.clone()),
        )
        .nest(
            "/api/v1/track",
            handlers::track::track_router(state.clone()),
        )
        .nest(
            "/api/v1/radio",
            handlers::radio::radio_router(state.clone()),
        )
        .split_for_parts();
    let router = router.merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", api));
    let router = router.layer(CookieManagerLayer::new());
    router
}
