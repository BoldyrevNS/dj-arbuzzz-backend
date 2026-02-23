use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use validator::Validate;

use crate::error::app_error::AppError;

pub mod auth;
pub mod track;

#[derive(Serialize)]
struct Res<T: Serialize> {
    status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

pub enum ApiResponse<T: Serialize> {
    OK(Option<T>),
    CREATED(Option<T>),
}

pub type ApiResult<T> = Result<ApiResponse<T>, AppError>;

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let (status, data) = match self {
            Self::OK(data) => (StatusCode::OK, data),
            Self::CREATED(data) => (StatusCode::CREATED, data),
        };
        let body = Json(json!(Res {
            status: status.as_u16(),
            data: data
        }));
        (status, body).into_response()
    }
}

pub struct ValidatedJSON<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJSON<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(data) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                let error_message = match rejection {
                    JsonRejection::JsonDataError(err) => {
                        format!("Ошибка данных JSON: {}", err.body_text())
                    }
                    JsonRejection::JsonSyntaxError(err) => {
                        format!("Синтаксическая ошибка JSON: {}", err)
                    }
                    JsonRejection::MissingJsonContentType(_) => {
                        "Отсутствует заголовок Content-Type: application/json".to_string()
                    }
                    _ => rejection.to_string(),
                };
                AppError::BadRequest(error_message, None).into_response()
            })?;

        data.validate()
            .map_err(|e| AppError::BadRequest(e.to_string(), None).into_response())?;

        Ok(ValidatedJSON(data))
    }
}
