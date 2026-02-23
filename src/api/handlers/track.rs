use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::middleware;
use serde::Deserialize;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;
use crate::api::handlers::{AuthData, auth_required};
use crate::dto::request::track::UserSelectTrackRequest;
use crate::dto::response::track::SearchTrackResponse;
use crate::dto::response::{ApiResponse, ApiResult, ValidatedJSON};
use crate::error::app_error::AppError;

pub fn track_router(app_state: Arc<AppState>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(search_track))
        .routes(routes!(select_track))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_required,
        ))
        .with_state(app_state)
}

#[derive(Deserialize)]
struct SearchTrackParams {
    track_name: Option<String>,
}

#[utoipa::path(
    get,
    path = "/search",
    tag = "Track",
    params(
        ("track_name" = Option<String>, Query, description = "Track name to search")
    ),
    responses(
        (status = 200, description = "Search tracks", body = SearchTrackResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
async fn search_track(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchTrackParams>,
) -> ApiResult<SearchTrackResponse> {
    let track_name = match params.track_name {
        Some(name) => name,
        None => {
            return Err(AppError::BadRequest(
                "Missing track_name parameter".to_string(),
                None,
            ));
        }
    };
    let res = state
        .services
        .track_service
        .search_track(track_name)
        .await?;

    Ok(ApiResponse::OK(Some(res)))
}

#[utoipa::path(
        post,
        path = "/select",
        tag = "Track",
        request_body = UserSelectTrackRequest,
        responses(
            (status = 200, description = "Success auth"),
            (status = 400, description = "Bad Request"),
            (status = 500, description = "Internal Server Error")
        )
    )]
#[axum_macros::debug_handler]
async fn select_track(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<Arc<AuthData>>,
    ValidatedJSON(payload): ValidatedJSON<UserSelectTrackRequest>,
) -> ApiResult<()> {
    state
        .services
        .track_service
        .user_select_track(session.user_id, payload)
        .await?;
    Ok(ApiResponse::OK(None))
}
