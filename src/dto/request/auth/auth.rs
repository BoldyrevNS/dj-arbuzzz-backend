#[derive(serde::Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct SignInRequest {
    #[validate(email(message = "Некорректный email"))]
    pub email: String,
    pub password: String,
}
