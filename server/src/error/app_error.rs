use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use deadpool::managed::BuildError;
use diesel_async::pooled_connection::deadpool::PoolError;
use redis::RedisError;
use serde_json::json;
use thiserror::Error;

use crate::error;
#[derive(Debug)]
pub enum ErrorCode {
    Unknown,
    OTPResendFailed,
    WrongOTP,
    OTPExpired,
    WrongOTPToken,
    UserAlreadyExists,
    OTPNotVerified,
    JWTExpired,
    ResendOTPTooManyRequests,
    SignUpFailed,
    JWTInvalid,
    TrackDurationLimit,
}

#[derive(serde::Serialize)]
struct AppErrorResponse {
    message: String,
    code: u16,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String, Option<ErrorCode>),

    #[error("Not found: {0}")]
    NotFound(String, Option<ErrorCode>),

    #[error("Validation error: {0}")]
    Validation(String, Option<ErrorCode>),

    #[error("Unauthorized: {0}")]
    Unauthorized(String, Option<ErrorCode>),

    #[error("Bad request: {0}")]
    BadRequest(String, Option<ErrorCode>),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Too many requests: {0}")]
    TooManyRequests(String, Option<ErrorCode>),
}

pub type AppResult<T> = Result<T, AppError>;

fn kek() -> AppResult<()> {
    if 1 > 0 {
        Ok(())
    } else {
        Err(AppError::Internal(anyhow::anyhow!("Error occurred")))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg, code) = match self {
            AppError::Database(msg, code) => (StatusCode::INTERNAL_SERVER_ERROR, msg, code),
            AppError::NotFound(msg, code) => (StatusCode::NOT_FOUND, msg, code),
            AppError::Validation(msg, code) => (StatusCode::BAD_REQUEST, msg, code),
            AppError::Unauthorized(msg, code) => (StatusCode::UNAUTHORIZED, msg, code),
            AppError::BadRequest(msg, code) => (StatusCode::BAD_REQUEST, msg, code),
            AppError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", err),
                None,
            ),
            AppError::TooManyRequests(msg, code) => (StatusCode::TOO_MANY_REQUESTS, msg, code),
        };
        let code_number = match code {
            Some(ErrorCode::OTPResendFailed) => 1001,
            Some(ErrorCode::WrongOTP) => 1002,
            Some(ErrorCode::WrongOTPToken) => 1003,
            Some(ErrorCode::OTPExpired) => 1004,
            Some(ErrorCode::UserAlreadyExists) => 1101,
            Some(ErrorCode::OTPNotVerified) => 1103,
            Some(ErrorCode::Unknown) => 1000,
            Some(ErrorCode::JWTExpired) => 1005,
            Some(ErrorCode::ResendOTPTooManyRequests) => 1006,
            Some(ErrorCode::JWTInvalid) => 1007,
            Some(ErrorCode::SignUpFailed) => 1102,
            Some(ErrorCode::TrackDurationLimit) => 1201,
            None => 1000,
        };
        let body = Json(json!({
            "status": status.as_u16(),
            "error": AppErrorResponse {
                message: msg,
                code: code_number,
            },
        }));
        (status, body).into_response()
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(value: diesel::result::Error) -> Self {
        AppError::Internal(anyhow::anyhow!(value.to_string()))
    }
}

impl From<RedisError> for AppError {
    fn from(value: RedisError) -> Self {
        AppError::Internal(anyhow::anyhow!(value.to_string()))
    }
}

impl From<BuildError> for AppError {
    fn from(value: BuildError) -> Self {
        AppError::Internal(anyhow::anyhow!(value.to_string()))
    }
}

impl From<PoolError> for AppError {
    fn from(value: PoolError) -> Self {
        AppError::Internal(anyhow::anyhow!(value.to_string()))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        AppError::Internal(anyhow::anyhow!("JWT error: {}", value))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::Internal(anyhow::anyhow!("Serialization error: {}", value))
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Internal(anyhow::anyhow!("IO error: {}", value))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        AppError::Internal(anyhow::anyhow!("HTTP request error: {}", value))
    }
}
