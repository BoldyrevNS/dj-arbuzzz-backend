use axum::body::Body;
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use std::sync::Arc;

use crate::error::app_error::AppResult;
use crate::{AppState, error::app_error::AppError};

pub mod auth;
pub mod radio;
pub mod sign_up;
pub mod track;

#[derive(Clone, Debug)]
pub struct AuthData {
    pub user_id: i32,
}

pub async fn auth_required(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> AppResult<Response> {
    let sid = state.services.auth_service.get_session_id_from_req(&req)?;

    let session_data = match state
        .services
        .auth_service
        .get_session_from_cache_and_update(sid)
        .await
    {
        Ok(session) => session,
        Err(_) => {
            return Err(AppError::Unauthorized(
                "Invalid session. Please sign in again.".to_string(),
                None,
            ));
        }
    };

    req.extensions_mut().insert(Arc::new(AuthData {
        user_id: session_data.user_id,
    }));
    let response = next.run(req).await;
    Ok(response)
}
