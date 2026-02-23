#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct StartRestoreResponse {
    hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_seconds: Option<u16>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ResendOTPResponse {
    pub hash: String,
    pub timeout_seconds: u16,
}
