use std::sync::Arc;

use axum::extract::State;

use crate::{
    AppState,
    dto::{
        request::auth::sign_up::{
            ResendOTPRequest, SignUpCompleteRequest, SignUpStartRequest, VerifyOTPRequest,
        },
        response::{
            ApiResponse, ApiResult, ValidatedJSON,
            auth::sign_up::{ResendOTPResponse, SignUpStartResponse},
        },
    },
};

#[utoipa::path(
        post,
        path = "/start",
        tag = "Sign Up",
        request_body = SignUpStartRequest,
        responses(
            (status = 200, description = "OTP sent successfully", body = SignUpStartResponse),
            (status = 400, description = "Bad Request"),
            (status = 500, description = "Internal Server Error")
        )
    )]
pub async fn start(
    State(state): State<Arc<AppState>>,
    ValidatedJSON(payload): ValidatedJSON<SignUpStartRequest>,
) -> ApiResult<SignUpStartResponse> {
    let res = state
        .services
        .sign_up_service
        .start_sign_up(payload)
        .await?;
    Ok(ApiResponse::OK(res))
}

#[utoipa::path(
        post,
        path = "/verify-otp",
        tag = "Sign Up",
        request_body = VerifyOTPRequest,
        responses(
            (status = 200, description = "OTP verified successfully"),
            (status = 400, description = "Bad Request"),
            (status = 500, description = "Internal Server Error")
        )
    )]
pub async fn verify(
    State(state): State<Arc<AppState>>,
    ValidatedJSON(payload): ValidatedJSON<VerifyOTPRequest>,
) -> ApiResult<()> {
    state.services.sign_up_service.verify_otp(payload).await?;
    Ok(ApiResponse::OK(()))
}

#[utoipa::path(
        post,
        path = "/resend-otp",
        tag = "Sign Up",
        request_body = ResendOTPRequest,
        responses(
            (status = 200, description = "OTP resent successfully", body = ResendOTPResponse),
            (status = 400, description = "Bad Request"),
            (status = 500, description = "Internal Server Error")
        )
    )]
pub async fn resend(
    State(state): State<Arc<AppState>>,
    ValidatedJSON(payload): ValidatedJSON<ResendOTPRequest>,
) -> ApiResult<ResendOTPResponse> {
    let res = state.services.sign_up_service.resend_otp(payload).await?;
    Ok(ApiResponse::OK(res))
}

#[utoipa::path(
        post,
        path = "/complete",
        tag = "Sign Up",
        request_body = SignUpCompleteRequest,
        responses(
            (status = 200, description = "Sign up completed successfully"),
            (status = 400, description = "Bad Request"),
            (status = 500, description = "Internal Server Error")
        )
    )]
pub async fn complete(
    State(state): State<Arc<AppState>>,
    ValidatedJSON(payload): ValidatedJSON<SignUpCompleteRequest>,
) -> ApiResult<()> {
    state
        .services
        .sign_up_service
        .sign_up_complete(payload)
        .await?;
    Ok(ApiResponse::OK(()))
}
