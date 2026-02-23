#[derive(serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct UserSelectTrackRequest {
    pub song_id: i32,
    pub owner_id: i32,
}
