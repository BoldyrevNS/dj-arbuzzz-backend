#[derive(serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct StartRestoreRequest {
    #[validate(email(message = "Неверный формат email"))]
    pub email: String,
}

#[derive(serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct VerifyOTPRequest {
    #[validate(email(message = "Неверный формат email"))]
    pub email: String,
    pub hash: String,
    #[validate(length(min = 6, max = 6, message = "OTP должен состоять из 6 символов"))]
    pub otp: String,
}

#[derive(serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct CompleteRestoreRequest {
    #[validate(email(message = "Неверный формат email"))]
    pub email: String,
    #[validate(length(min = 8, message = "Пароль должен быть не менее 8 символов"))]
    pub password: String,
    pub hash: String,
}
