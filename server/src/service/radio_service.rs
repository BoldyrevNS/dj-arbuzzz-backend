use std::{fs, sync::Arc, time::Instant};

use axum::body::Bytes;
use tokio::{
    io::AsyncReadExt,
    sync::{broadcast, Notify, RwLock},
};

use crate::{
    config::AppConfig,
    dto::response::{
        raido::GetCurrentTrackResponse,
        websocket::{CurrentTrackData, WebSocketMessage},
    },
    error::app_error::AppResult,
    infrastucture::repositories::track_repository::TrackRepository,
    service::playlist_service::{PlaylistItem, PlaylistService},
};

const CHUNK_MS: u64 = 100;
const BROADCAST_CAPACITY: usize = 256;
const DFPWM_BROADCAST_CAPACITY: usize = 2048; // Больше буфер для DFPWM (256 секунд @ 125ms/чанк)
const WS_EVENT_CAPACITY: usize = 100;

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
    dfpwm_sender: broadcast::Sender<Bytes>,
    ws_event_sender: broadcast::Sender<WebSocketMessage>,
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
        let (dfpwm_sender, _) = broadcast::channel(DFPWM_BROADCAST_CAPACITY);
        let (ws_event_sender, _) = broadcast::channel(WS_EVENT_CAPACITY);
        let service = Arc::new(RadioService {
            sender,
            dfpwm_sender,
            ws_event_sender,
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

        // Forward playlist events through radio service
        let svc = service.clone();
        tokio::spawn(async move {
            let mut rx = svc.playlist_service.subscribe_events();
            while let Ok(msg) = rx.recv().await {
                let _ = svc.ws_event_sender.send(msg);
            }
        });

        service
            .create_songs_dir_if_not_exists()
            .expect("Failed to create songs directory");
        service
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Bytes> {
        self.sender.subscribe()
    }

    pub fn subscribe_dfpwm(&self) -> broadcast::Receiver<Bytes> {
        self.dfpwm_sender.subscribe()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<WebSocketMessage> {
        self.ws_event_sender.subscribe()
    }

    pub async fn get_current_track_ws(&self) -> AppResult<CurrentTrackData> {
        let state = self.state.read().await;
        let name = if let Some(current_track) = &state.current_track {
            Some(format!(
                "{} - {}",
                current_track.item.artist, current_track.item.title
            ))
        } else {
            None
        };
        Ok(CurrentTrackData { name })
    }

    fn notify_current_track_changed(&self, name: Option<String>) {
        let msg = WebSocketMessage::CurrentTrack(CurrentTrackData { name });
        let _ = self.ws_event_sender.send(msg);
    }

    pub fn create_songs_dir_if_not_exists(&self) -> AppResult<()> {
        let path = std::path::Path::new(self.config.songs_config.songs_dir_path.as_str());
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    pub async fn get_current_track(&self) -> GetCurrentTrackResponse {
        let state = self.state.read().await;
        if let Some(current_track) = &state.current_track {
            GetCurrentTrackResponse {
                name: Some(format!(
                    "{} - {}",
                    current_track.item.artist, current_track.item.title
                )),
            }
        } else {
            GetCurrentTrackResponse { name: None }
        }
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

            // Notify WebSocket clients about track change
            self.notify_current_track_changed(Some(format!("{} - {}", item.artist, item.title)));

            if is_auto {
                let notified = self.queue_notify.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();

                let mp3_path = file_path.clone();
                let dfpwm_path = file_path.clone();

                tokio::select! {
                    _ = async {
                        let (mp3_result, dfpwm_result) = tokio::join!(
                            self.stream_file(&mp3_path, chunk_size, bytes_per_sec),
                            self.stream_file_dfpwm(&dfpwm_path, bytes_per_sec)
                        );
                        if let Err(e) = mp3_result {
                            eprintln!("[radio] Error streaming MP3 {}: {}", mp3_path, e);
                        }
                        if let Err(e) = dfpwm_result {
                            eprintln!("[radio] Error streaming DFPWM {}: {}", dfpwm_path, e);
                        }
                    } => {}
                    _ = notified => {
                    }
                }
            } else {
                let (mp3_result, dfpwm_result) = tokio::join!(
                    self.stream_file(&file_path, chunk_size, bytes_per_sec),
                    self.stream_file_dfpwm(&file_path, bytes_per_sec)
                );
                if let Err(e) = mp3_result {
                    eprintln!("[radio] Error streaming MP3 {}: {}", file_path, e);
                }
                if let Err(e) = dfpwm_result {
                    eprintln!("[radio] Error streaming DFPWM {}: {}", file_path, e);
                }
            }

            {
                self.state.write().await.current_track = None;
            }

            // Notify WebSocket clients that track ended
            self.notify_current_track_changed(None);
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

    async fn stream_file_dfpwm(&self, file_path: &str, _bytes_per_sec: u64) -> AppResult<()> {
        use symphonia::core::audio::SampleBuffer;
        use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(file_path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("mp3");

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| anyhow::anyhow!("Failed to probe format: {}", e))?;

        let mut format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow::anyhow!("No supported audio tracks"))?;

        let dec_opts: DecoderOptions = Default::default();
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .map_err(|e| anyhow::anyhow!("Failed to create decoder: {}", e))?;

        let track_id = track.id;
        let mut dfpwm_encoder = crate::service::dfpwm::DfpwmEncoder::new();

        // DFPWM requires 48kHz mono signed 8-bit PCM
        let mut sample_buf: Option<SampleBuffer<i16>> = None;
        let mut saved_spec = None;
        let mut resampled_buf = Vec::new();

        let chunk_size = 6144; // 128ms at 48kHz (кратно 8 для DFPWM)
        let mut output_buffer = Vec::with_capacity(chunk_size / 8);

        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(_) => break,
            };

            if packet.track_id() != track_id {
                continue;
            }

            let decoded = match decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(_) => continue,
            };

            if sample_buf.is_none() {
                let spec = *decoded.spec();
                saved_spec = Some(spec);
                let duration = decoded.capacity() as u64;
                sample_buf = Some(SampleBuffer::<i16>::new(duration, spec));
            }

            if let Some(ref mut buf) = sample_buf {
                buf.copy_interleaved_ref(decoded);

                let samples = buf.samples();
                let spec = saved_spec.as_ref().unwrap();

                // Convert to mono if needed
                let mono_samples: Vec<i16> = if spec.channels.count() > 1 {
                    samples
                        .chunks(spec.channels.count())
                        .map(|chunk| {
                            let sum: i32 = chunk.iter().map(|&s| s as i32).sum();
                            (sum / chunk.len() as i32) as i16
                        })
                        .collect()
                } else {
                    samples.to_vec()
                };

                // Resample to 48kHz if needed using cubic interpolation
                let target_rate = 48000;
                let source_rate = spec.rate;

                let resampled = if source_rate != target_rate {
                    let ratio = target_rate as f32 / source_rate as f32;
                    let new_len = (mono_samples.len() as f32 * ratio) as usize;
                    (0..new_len)
                        .map(|i| {
                            let pos = i as f32 / ratio;
                            let idx = pos as usize;
                            let frac = pos - idx as f32;

                            // Cubic (Catmull-Rom) interpolation для лучшего качества
                            let len = mono_samples.len();
                            if idx == 0 || idx >= len - 2 {
                                // Граничные случаи: линейная интерполяция
                                if idx >= len - 1 {
                                    mono_samples[len - 1]
                                } else {
                                    let a = mono_samples[idx] as f32;
                                    let b = mono_samples[idx + 1] as f32;
                                    (a + (b - a) * frac) as i16
                                }
                            } else {
                                // Cubic interpolation
                                let y0 = mono_samples[idx - 1] as f32;
                                let y1 = mono_samples[idx] as f32;
                                let y2 = mono_samples[idx + 1] as f32;
                                let y3 = mono_samples[idx + 2] as f32;

                                let a0 = y3 - y2 - y0 + y1;
                                let a1 = y0 - y1 - a0;
                                let a2 = y2 - y0;
                                let a3 = y1;

                                let result =
                                    a0 * frac * frac * frac + a1 * frac * frac + a2 * frac + a3;
                                result.clamp(-32768.0, 32767.0) as i16
                            }
                        })
                        .collect()
                } else {
                    mono_samples
                };

                // Нормализация громкости для лучшего использования динамического диапазона
                let max_amplitude = resampled.iter().map(|&s| s.abs()).max().unwrap_or(1) as f32;

                let normalization_factor = if max_amplitude > 16384.0 {
                    // Если сигнал слишком громкий, немного уменьшаем
                    16384.0 / max_amplitude
                } else if max_amplitude < 8192.0 && max_amplitude > 0.0 {
                    // Если слишком тихий, немного увеличиваем (но не более чем в 2 раза)
                    (16384.0 / max_amplitude).min(2.0)
                } else {
                    1.0
                };

                // Convert to signed 8-bit with proper scaling and dithering
                for &sample in &resampled {
                    // Применяем нормализацию и конвертируем i16 -> i8
                    let normalized = (sample as f32 * normalization_factor) as i32;
                    let sample_8bit = ((normalized + 128) / 256).clamp(-128, 127) as i8;
                    resampled_buf.push(sample_8bit);
                }

                // Encode to DFPWM in chunks
                while resampled_buf.len() >= chunk_size {
                    let chunk: Vec<i8> = resampled_buf.drain(..chunk_size).collect();

                    output_buffer.clear();
                    dfpwm_encoder.encode(&chunk, &mut output_buffer);

                    let _ = self
                        .dfpwm_sender
                        .send(Bytes::copy_from_slice(&output_buffer));

                    // Timing: chunk_size samples at 48kHz
                    let duration_ms = (chunk_size * 1000) / 48000;
                    tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms as u64))
                        .await;
                }
            }
        }

        // Flush remaining samples
        if !resampled_buf.is_empty() {
            output_buffer.clear();
            dfpwm_encoder.encode(&resampled_buf, &mut output_buffer);
            let _ = self
                .dfpwm_sender
                .send(Bytes::copy_from_slice(&output_buffer));
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
