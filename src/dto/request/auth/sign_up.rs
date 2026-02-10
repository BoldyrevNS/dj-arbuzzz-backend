#[derive(serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct SignUpStartRequest {
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
pub struct ResendOTPRequest {
    #[validate(email(message = "Неверный формат email"))]
    pub email: String,

    pub hash: String,
}

#[derive(serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct SignUpCompleteRequest {
    #[validate(email(message = "Неверный формат email"))]
    pub email: String,

    #[validate(length(
        min = 3,
        max = 20,
        message = "Имя пользователя должно быть от 3 до 20 символов"
    ))]
    pub username: String,

    #[validate(length(min = 8, message = "Пароль должен быть не менее 8 символов"))]
    pub password: String,

    pub hash: String,
}
