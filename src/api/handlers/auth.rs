use std::sync::Arc;

use axum::{body::Body, extract::State, http::Request};
use tower_cookies::Cookies;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    AppState,
    dto::{
        request::auth::auth::SignInRequest,
        response::{ApiResponse, ApiResult, ValidatedJSON},
    },
};

pub fn auth_router(app_state: Arc<AppState>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(sign_in))
        .routes(routes!(logout))
        .with_state(app_state)
}

#[utoipa::path(
        post,
        path = "/sign-in",
        tag = "Auth",
        request_body = SignInRequest,
        responses(
            (status = 200, description = "Success auth"),
            (status = 400, description = "Bad Request"),
            (status = 500, description = "Internal Server Error")
        )
    )]
async fn sign_in(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    ValidatedJSON(payload): ValidatedJSON<SignInRequest>,
) -> ApiResult<()> {
    state
        .services
        .auth_service
        .sign_in(payload, cookies)
        .await?;
    Ok(ApiResponse::OK(None))
}

#[utoipa::path(
        post,
        path = "/logout",
        tag = "Auth",
        responses(
            (status = 200, description = "Success logout"),
            (status = 500, description = "Internal Server Error")
        )
    )]
#[axum_macros::debug_handler]
async fn logout(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    req: Request<Body>,
) -> ApiResult<()> {
    let sid = match state.services.auth_service.get_session_id_from_req(&req) {
        Ok(sid) => sid,
        Err(_) => return Ok(ApiResponse::OK(None)),
    };
    state
        .services
        .auth_service
        .delete_session(cookies, sid)
        .await?;
    Ok(ApiResponse::OK(None))
}
