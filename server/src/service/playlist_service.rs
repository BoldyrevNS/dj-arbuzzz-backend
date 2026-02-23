use redis::AsyncCommands;
use std::sync::Arc;

use crate::{
    error::app_error::AppResult,
    infrastucture::cache::{client::Cache, keys::AppCacheKey},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlaylistItem {
    pub id: i32,
    pub song_id: i32,
    pub owner_id: i32,
    pub artist: String,
    pub title: String,
    pub duration_sec: i32,
    pub download_url: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Playlist {
    pub items: Vec<PlaylistItem>,
}

pub struct PlaylistService {
    cache: Arc<Cache>,
}

impl PlaylistService {
    pub fn new(cache: Arc<Cache>) -> Self {
        PlaylistService { cache }
    }

    pub async fn add_new_track(&self, item: PlaylistItem) -> AppResult<()> {
        let key = AppCacheKey::PLAYLIST().build_key();
        let mut con = self.cache.get_async_conn().await?;
        let playlist = match con.get::<String, String>(key.clone()).await {
            Ok(playlist_str) => serde_json::from_str::<Playlist>(&playlist_str)?,
            Err(_) => Playlist { items: vec![] },
        };
        let mut new_playlist = playlist;
        new_playlist.items.push(item);
        let playlist_str = serde_json::to_string(&new_playlist)?;
        let _: () = con.set(key, playlist_str).await?;
        Ok(())
    }

    pub async fn pop_track(&self) -> AppResult<PlaylistItem> {
        let key = AppCacheKey::PLAYLIST().build_key();
        let mut con = self.cache.get_async_conn().await?;
        let playlist = match con.get::<String, String>(key.clone()).await {
            Ok(playlist_str) => serde_json::from_str::<Playlist>(&playlist_str)?,
            Err(_) => Playlist { items: vec![] },
        };
        let mut new_playlist = playlist;
        if new_playlist.items.is_empty() {
            return Err(crate::error::app_error::AppError::NotFound(
                "Playlist is empty".to_string(),
                None,
            ));
        }
        let item = new_playlist.items.remove(0);
        let playlist_str = serde_json::to_string(&new_playlist)?;
        let _: () = con.set(key, playlist_str).await?;
        Ok(item)
    }

    pub async fn get_playlist(&self) -> AppResult<Playlist> {
        let key = AppCacheKey::PLAYLIST().build_key();
        let mut con = self.cache.get_async_conn().await?;
        let playlist = match con.get::<String, String>(key).await {
            Ok(playlist_str) => serde_json::from_str::<Playlist>(&playlist_str)?,
            Err(_) => Playlist { items: vec![] },
        };
        Ok(playlist)
    }
}
