use std::sync::Arc;

use tokio::sync::Notify;

use crate::{
    dto::{request::track::UserSelectTrackRequest, response::track::SearchTrackResponse},
    error::app_error::{AppError, AppResult, ErrorCode},
    infrastucture::{database::models::NewTrack, repositories::track_repository::TrackRepository},
    service::playlist_service::{PlaylistItem, PlaylistService},
};

#[derive(serde::Deserialize, Debug, Clone)]
struct TrackFromApi {
    id: i32,
    owner_id: i32,
    artist: String,
    title: String,
    duration: i32,
    date: i64,
    url: String,
}

#[derive(serde::Deserialize, Debug)]
struct SearchTrackInApiItems {
    items: Vec<TrackFromApi>,
}
#[derive(serde::Deserialize, Debug)]
struct SearchTrackByNameInApiResponse {
    response: SearchTrackInApiItems,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct SearchTrackByIdInApiResponse {
    response: Vec<TrackFromApi>,
}

pub struct TrackService {
    track_repository: Arc<TrackRepository>,
    playlist_service: Arc<PlaylistService>,
    config: Arc<crate::config::AppConfig>,
    queue_notify: Arc<Notify>,
}

impl TrackService {
    pub fn new(
        track_repository: Arc<TrackRepository>,
        playlist_service: Arc<PlaylistService>,
        config: Arc<crate::config::AppConfig>,
        queue_notify: Arc<Notify>,
    ) -> Self {
        TrackService {
            track_repository,
            playlist_service,
            config,
            queue_notify,
        }
    }

    pub async fn search_track(&self, search_value: String) -> AppResult<SearchTrackResponse> {
        let search_response = self.search_track_by_name_in_api(&search_value).await?;

        let tracks = search_response
            .response
            .items
            .into_iter()
            .map(|item| crate::dto::response::track::SearchTrackDTO {
                song_id: item.id,
                owner_id: item.owner_id,
                duration: item.duration,
                name: format!("{} - {}", item.artist, item.title),
            })
            .collect();

        Ok(SearchTrackResponse { tracks })
    }

    pub async fn user_select_track(
        &self,
        user_id: i32,
        data: UserSelectTrackRequest,
    ) -> AppResult<()> {
        let tracks = self
            .search_track_by_id_in_api(data.song_id, data.owner_id)
            .await?;
        if tracks.response.is_empty() {
            return Err(AppError::BadRequest(
                "Track not found in music API".to_string(),
                None,
            ));
        }

        let track = tracks.response[0].clone();
        if track.duration > 60 * 10 {
            return Err(AppError::BadRequest(
                "Track duration limit".to_string(),
                Some(ErrorCode::TrackDurationLimit),
            ));
        }

        let (track, _) = self
            .track_repository
            .create_track_with_user_track(
                &NewTrack {
                    song_id: track.id,
                    owner_id: track.owner_id,
                    artist: track.artist,
                    title: track.title,
                    duration_sec: track.duration,
                    download_url: track.url,
                    likes_count: None,
                    listens_count: None,
                },
                user_id,
            )
            .await?;
        self.playlist_service
            .add_new_track(PlaylistItem {
                id: track.id,
                song_id: track.song_id,
                owner_id: track.owner_id,
                artist: track.artist,
                title: track.title,
                duration_sec: track.duration_sec,
                download_url: track.download_url,
            })
            .await?;
        self.queue_notify.notify_one();
        Ok(())
    }

    async fn search_track_by_name_in_api(
        &self,
        search_value: &str,
    ) -> AppResult<SearchTrackByNameInApiResponse> {
        let url = reqwest::Url::parse_with_params(
            format!("{}.search", self.config.music_api_url.as_str()).as_str(),
            &[
                ("q", search_value),
                ("access_token", self.config.music_api_token.as_str()),
                ("v", "5.131"),
            ],
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        let search_response = response
            .json::<SearchTrackByNameInApiResponse>()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        Ok(search_response)
    }

    async fn search_track_by_id_in_api(
        &self,
        song_id_val: i32,
        owner_id_val: i32,
    ) -> AppResult<SearchTrackByIdInApiResponse> {
        let url = reqwest::Url::parse_with_params(
            format!("{}.getById", self.config.music_api_url.as_str()).as_str(),
            &[
                (
                    "audios",
                    format!("{}_{}", owner_id_val, song_id_val).as_str(),
                ),
                ("access_token", self.config.music_api_token.as_str()),
                ("v", "5.131"),
            ],
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        println!("{}", url);
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        let search_response = response
            .json::<SearchTrackByIdInApiResponse>()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        Ok(search_response)
    }
}
