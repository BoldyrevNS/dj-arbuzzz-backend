use std::{fs, sync::Arc, time::Instant};

use axum::body::Bytes;
use tokio::{
    io::AsyncReadExt,
    sync::{Notify, RwLock, broadcast},
};

use crate::{
    config::AppConfig,
    error::app_error::AppResult,
    infrastucture::repositories::track_repository::TrackRepository,
    service::playlist_service::{PlaylistItem, PlaylistService},
};

const CHUNK_MS: u64 = 100;
const BROADCAST_CAPACITY: usize = 256;

enum NextTrack {
    Queued(PlaylistItem),
    Auto(PlaylistItem),
}

pub struct CurrentTrack {
    pub item: PlaylistItem,
    pub started_at: Instant,
    pub file_size: u64,
}

pub struct RadioState {
    pub current_track: Option<CurrentTrack>,
}

pub struct RadioService {
    sender: broadcast::Sender<Bytes>,
    pub state: Arc<RwLock<RadioState>>,
    playlist_service: Arc<PlaylistService>,
    track_repository: Arc<TrackRepository>,
    config: Arc<AppConfig>,
    queue_notify: Arc<Notify>,
}

impl RadioService {
    pub fn new(
        playlist_service: Arc<PlaylistService>,
        track_repository: Arc<TrackRepository>,
        config: Arc<AppConfig>,
        queue_notify: Arc<Notify>,
    ) -> Arc<Self> {
        let (sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        let service = Arc::new(RadioService {
            sender,
            state: Arc::new(RwLock::new(RadioState {
                current_track: None,
            })),
            playlist_service,
            track_repository,
            config,
            queue_notify,
        });

        let svc = service.clone();
        tokio::spawn(async move {
            svc.run_broadcaster().await;
        });

        service
            .create_songs_dir_if_not_exists()
            .expect("Failed to create songs directory");
        service
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Bytes> {
        self.sender.subscribe()
    }

    pub fn create_songs_dir_if_not_exists(&self) -> AppResult<()> {
        let path = std::path::Path::new(self.config.songs_config.songs_dir_path.as_str());
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    async fn download_track(&self, song_id: i32, owner_id: i32, url: String) -> AppResult<()> {
        let file_name = format!("{}_{}.mp3", owner_id, song_id);
        let file_path = format!("{}/{}", self.config.songs_config.songs_dir_path, file_name);
        if !std::path::Path::new(&file_path).exists() {
            let response = reqwest::get(url).await?;
            let bytes = response.bytes().await?;
            fs::write(file_path, bytes)?;
        }
        Ok(())
    }

    async fn run_broadcaster(&self) {
        loop {
            let next = match self.next_track_item().await {
                Ok(next) => next,
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            };

            let (item, is_auto) = match next {
                NextTrack::Queued(item) => (item, false),
                NextTrack::Auto(item) => (item, true),
            };

            if let Err(e) = self
                .download_track(item.song_id, item.owner_id, item.download_url.clone())
                .await
            {
                eprintln!("[radio] Failed to download track {}: {}", item.id, e);
                continue;
            }

            let file_name = format!("{}_{}.mp3", item.owner_id, item.song_id);
            let file_path = format!("{}/{}", self.config.songs_config.songs_dir_path, file_name);

            let file_size: u64 = match tokio::fs::metadata(&file_path).await {
                Ok(m) => m.len(),
                Err(e) => {
                    eprintln!("[radio] Cannot stat {}: {}", file_path, e);
                    continue;
                }
            };

            let duration_secs = item.duration_sec as u64;
            let bytes_per_sec = if duration_secs > 0 {
                file_size / duration_secs
            } else {
                16_000
            };
            let chunk_size = ((bytes_per_sec * CHUNK_MS) / 1000).max(1024) as usize;

            {
                let mut state = self.state.write().await;
                state.current_track = Some(CurrentTrack {
                    item: item.clone(),
                    started_at: Instant::now(),
                    file_size,
                });
            }

            if is_auto {
                let notified = self.queue_notify.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();

                tokio::select! {
                    result = self.stream_file(&file_path, chunk_size, bytes_per_sec) => {
                        if let Err(e) = result {
                            eprintln!("[radio] Error streaming {}: {}", file_path, e);
                        }
                    }
                    _ = notified => {
                    }
                }
            } else {
                if let Err(e) = self
                    .stream_file(&file_path, chunk_size, bytes_per_sec)
                    .await
                {
                    eprintln!("[radio] Error streaming {}: {}", file_path, e);
                }
            }

            {
                self.state.write().await.current_track = None;
            }
        }
    }

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
                break;
            }

            let _ = self.sender.send(Bytes::copy_from_slice(&buf[..n]));

            let chunk_duration_ms = if bytes_per_sec > 0 {
                (n as u64 * 1000) / bytes_per_sec
            } else {
                CHUNK_MS
            };
            let sleep_dur = tokio::time::Duration::from_millis(chunk_duration_ms)
                .saturating_sub(read_start.elapsed());
            if !sleep_dur.is_zero() {
                tokio::time::sleep(sleep_dur).await;
            }
        }

        Ok(())
    }

    async fn next_track_item(&self) -> AppResult<NextTrack> {
        if let Ok(item) = self.playlist_service.pop_track().await {
            return Ok(NextTrack::Queued(item));
        }

        let track = self.track_repository.find_random_track().await?;
        let item = PlaylistItem {
            id: track.id,
            song_id: track.song_id,
            owner_id: track.owner_id,
            artist: track.artist,
            title: track.title,
            duration_sec: track.duration_sec,
            download_url: track.download_url,
        };
        Ok(NextTrack::Auto(item))
    }
}
