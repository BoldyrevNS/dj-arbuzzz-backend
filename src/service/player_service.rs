use std::{fs, sync::Arc};

use crate::{
    config::AppConfig, error::app_error::AppResult, service::playlist_service::PlaylistService,
};

pub struct PlayerService {
    config: Arc<AppConfig>,
    playlist_service: Arc<PlaylistService>,
}

impl PlayerService {
    pub fn new(config: Arc<AppConfig>, playlist_service: Arc<PlaylistService>) -> Self {
        PlayerService {
            config,
            playlist_service,
        }
    }

    pub fn create_songs_dir_if_not_exists(&self) -> AppResult<()> {
        let path = std::path::Path::new(self.config.songs_config.songs_dir_path.as_str());
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    pub async fn download_track(&self, song_id: i32, owner_id: i32, url: String) -> AppResult<()> {
        let file_name = format!("{}_{}.mp3", owner_id, song_id);
        let file_path = format!("{}/{}", self.config.songs_config.songs_dir_path, file_name);
        if !std::path::Path::new(&file_path).exists() {
            let response = reqwest::get(url).await?;
            let bytes = response.bytes().await?;
            fs::write(file_path, bytes)?;
        }
        Ok(())
    }
}
