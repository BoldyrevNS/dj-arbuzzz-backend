#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SearchTrackDTO {
    pub name: String,
    pub duration: i32,
    pub song_id: i32,
    pub owner_id: i32,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SearchTrackResponse {
    pub tracks: Vec<SearchTrackDTO>,
}
