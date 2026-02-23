#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SignUpStartResponse {
    pub token: String,
    pub timeout_seconds: u16,
}

#[derive(serde::Serialize, utoipa::ToSchema)]

pub struct ResendOTPResponse {
    pub token: String,
    pub timeout_seconds: u16,
}
