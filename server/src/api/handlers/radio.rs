use std::{convert::Infallible, sync::Arc};

use axum::{
    body::{Body, Bytes},
    extract::State,
    response::Response,
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    dto::response::{raido::GetCurrentTrackResponse, ApiResponse, ApiResult},
    AppState,
};

pub fn radio_router(app_state: Arc<AppState>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(stream_radio))
        .routes(routes!(get_current_track))
        .with_state(app_state)
}

#[utoipa::path(
    get,
    path = "/stream",
    tag = "Radio",
    responses(
        (status = 200, description = "Live audio stream (audio/mpeg)", content_type = "audio/mpeg"),
        (status = 503, description = "No tracks available yet")
    )
)]
async fn stream_radio(State(state): State<Arc<AppState>>) -> Response {
    let receiver = state.services.radio_service.subscribe();

    let stream = BroadcastStream::new(receiver)
        .filter_map(|result| result.ok().map(|bytes| Ok::<Bytes, Infallible>(bytes)));

    Response::builder()
        .status(200)
        .header("Content-Type", "audio/mpeg")
        .header("Cache-Control", "no-cache, no-store")
        .header("Transfer-Encoding", "chunked")
        .body(Body::from_stream(stream))
        .unwrap()
}

#[utoipa::path(
        get,
        path = "/current-track",
        tag = "Radio",
        responses(
            (status = 200, description = "Success", body = GetCurrentTrackResponse),
            (status = 400, description = "Bad Request"),
            (status = 500, description = "Internal Server Error")
        )
    )]
async fn get_current_track(
    State(state): State<Arc<AppState>>,
) -> ApiResult<GetCurrentTrackResponse> {
    let res = state.services.radio_service.get_current_track().await;
    Ok(ApiResponse::OK(Some(res)))
}
