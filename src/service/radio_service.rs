use std::{sync::Arc, time::Instant};

use axum::body::Bytes;
use rand::seq::SliceRandom;
use tokio::{
    io::AsyncReadExt,
    sync::{broadcast, RwLock},
};

use crate::{
    config::AppConfig,
    error::app_error::{AppError, AppResult},
    service::{
        player_service::PlayerService,
        playlist_service::{PlaylistItem, PlaylistService},
    },
};

/// How many milliseconds of audio each broadcast chunk represents (used for throttling).
const CHUNK_MS: u64 = 100;
/// Broadcast channel capacity (number of buffered chunks).
const BROADCAST_CAPACITY: usize = 256;

pub struct CurrentTrack {
    pub item: PlaylistItem,
    pub file_path: String,
    pub started_at: Instant,
    pub file_size: u64,
}

pub struct RadioState {
    pub current_track: Option<CurrentTrack>,
    /// History of all tracks that have been played, used when the playlist is exhausted.
    pub played_tracks: Vec<PlaylistItem>,
}

pub struct RadioService {
    sender: broadcast::Sender<Bytes>,
    pub state: Arc<RwLock<RadioState>>,
    playlist_service: Arc<PlaylistService>,
    player_service: Arc<PlayerService>,
    config: Arc<AppConfig>,
}

impl RadioService {
    /// Create the service and immediately launch the background broadcaster task.
    pub fn new(
        playlist_service: Arc<PlaylistService>,
        player_service: Arc<PlayerService>,
        config: Arc<AppConfig>,
    ) -> Arc<Self> {
        let (sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        let service = Arc::new(RadioService {
            sender,
            state: Arc::new(RwLock::new(RadioState {
                current_track: None,
                played_tracks: vec![],
            })),
            playlist_service,
            player_service,
            config,
        });

        let svc = service.clone();
        tokio::spawn(async move {
            svc.run_broadcaster().await;
        });

        service
    }

    /// Subscribe to the live audio broadcast. Each subscriber receives the same
    /// `Bytes` chunks in real time, starting from the moment they subscribe.
    pub fn subscribe(&self) -> broadcast::Receiver<Bytes> {
        self.sender.subscribe()
    }

    /// Background task: continuously reads tracks and broadcasts their audio bytes
    /// to all subscribers at real-time rate.
    async fn run_broadcaster(&self) {
        loop {
            let item = match self.next_track_item().await {
                Ok(item) => item,
                Err(_) => {
                    // No tracks available – wait and retry.
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            };

            // Download the track file if it is not already on disk.
            if let Err(e) = self
                .player_service
                .download_track(item.song_id, item.owner_id, item.download_url.clone())
                .await
            {
                eprintln!("[radio] Failed to download track {}: {}", item.id, e);
                continue;
            }

            let file_name = format!("{}_{}.mp3", item.owner_id, item.song_id);
            let file_path = format!(
                "{}/{}",
                self.config.songs_config.songs_dir_path, file_name
            );

            let file_size: u64 = match tokio::fs::metadata(&file_path).await {
                Ok(m) => m.len(),
                Err(e) => {
                    eprintln!("[radio] Cannot stat {}: {}", file_path, e);
                    continue;
                }
            };

            // Calculate chunk size to achieve real-time playback rate.
            let duration_secs = item.duration_sec as u64;
            let bytes_per_sec = if duration_secs > 0 {
                file_size / duration_secs
            } else {
                16_000 // fall back to 128 kbps
            };
            let chunk_size =
                ((bytes_per_sec * CHUNK_MS) / 1000).max(1024) as usize;

            // Update the shared state so that new subscribers know what is playing.
            {
                let mut state = self.state.write().await;
                state.current_track = Some(CurrentTrack {
                    item: item.clone(),
                    file_path: file_path.clone(),
                    started_at: Instant::now(),
                    file_size,
                });
                state.played_tracks.push(item.clone());
            }

            // Stream the file, throttled to real-time.
            if let Err(e) = self
                .stream_file(&file_path, chunk_size, bytes_per_sec)
                .await
            {
                eprintln!("[radio] Error streaming {}: {}", file_path, e);
            }

            // Track finished – clear current track so next iteration picks a new one.
            {
                let mut state = self.state.write().await;
                state.current_track = None;
            }
        }
    }

    /// Read a file in chunks, broadcasting each chunk and throttling to `bytes_per_sec`.
    async fn stream_file(
        &self,
        file_path: &str,
        chunk_size: usize,
        bytes_per_sec: u64,
    ) -> AppResult<()> {
        let mut file: tokio::fs::File = tokio::fs::File::open(file_path).await?;
        let mut buf = vec![0u8; chunk_size];

        loop {
            let read_start = tokio::time::Instant::now();

            let n = file.read(&mut buf).await?;
            if n == 0 {
                break; // EOF
            }

            // Broadcast – ignore errors when there are no active subscribers.
            let _ = self.sender.send(Bytes::copy_from_slice(&buf[..n]));

            // Sleep for the time this chunk would take to play to maintain real-time rate.
            let chunk_duration_ms = if bytes_per_sec > 0 {
                (n as u64 * 1000) / bytes_per_sec
            } else {
                CHUNK_MS
            };
            let sleep_dur =
                tokio::time::Duration::from_millis(chunk_duration_ms)
                    .saturating_sub(read_start.elapsed());
            if !sleep_dur.is_zero() {
                tokio::time::sleep(sleep_dur).await;
            }
        }

        Ok(())
    }

    /// Choose the next track: first from the playlist queue, then a random track
    /// from the played-track history.
    async fn next_track_item(&self) -> AppResult<PlaylistItem> {
        if let Ok(item) = self.playlist_service.pop_track().await {
            return Ok(item);
        }

        let state = self.state.read().await;
        if state.played_tracks.is_empty() {
            return Err(AppError::NotFound(
                "No tracks available for radio".to_string(),
                None,
            ));
        }

        let mut rng = rand::thread_rng();
        let item = state
            .played_tracks
            .choose(&mut rng)
            .cloned()
            .expect("played_tracks is non-empty, choose should always return Some");
        Ok(item)
    }
}
