pub mod handlers;
pub mod routes;

use std::sync::Arc;

use tokio::sync::Notify;

use crate::{
    config::AppConfig,
    infrastucture::{
        cache::client::Cache,
        database::pool::DbPool,
        repositories::{track_repository::TrackRepository, users_repository::UsersRepository},
    },
    service::{
        auth::{
            auth_service::AuthService, restore_service::RestoreService,
            sign_up_service::SignUpService,
        },
        otp_service::OTPService,
        playlist_service::PlaylistService,
        radio_service::RadioService,
        smtp_service::SMTPService,
        token_service::TokenService,
        track_service::TrackService,
    },
};

pub struct Services {
    pub sign_up_service: Arc<SignUpService>,
    pub restore_service: Arc<RestoreService>,
    pub auth_service: Arc<AuthService>,
    pub track_service: Arc<TrackService>,
    pub playlist_service: Arc<PlaylistService>,
    pub radio_service: Arc<RadioService>,
}

pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub config: Arc<AppConfig>,
    pub services: Services,
}

impl AppState {
    pub fn new(config: AppConfig, db_pool: DbPool) -> Self {
        let config = Arc::new(config);

        // Shared services
        let otp_service = Arc::new(OTPService::new());
        let smtp_service = Arc::new(SMTPService::new(&config));
        let token_service = Arc::new(TokenService::new(config.clone()));

        let cache = Arc::new(Cache::new(&config.redis_config.url));
        let db_pool = Arc::new(db_pool);

        let users_repository = Arc::new(UsersRepository::new(db_pool.clone()));
        let track_repository = Arc::new(TrackRepository::new(db_pool.clone()));

        let playlist_service = Arc::new(PlaylistService::new(cache.clone()));

        let sign_up_service = Arc::new(SignUpService::new(
            cache.clone(),
            otp_service.clone(),
            smtp_service.clone(),
            users_repository.clone(),
            token_service.clone(),
        ));

        let restore_service = Arc::new(RestoreService::new(
            cache.clone(),
            otp_service.clone(),
            smtp_service.clone(),
            users_repository.clone(),
            token_service.clone(),
        ));

        // Shared notify used to interrupt auto-play when a track is queued.
        let queue_notify = Arc::new(Notify::new());

        let track_service = Arc::new(TrackService::new(
            track_repository.clone(),
            playlist_service.clone(),
            config.clone(),
            queue_notify.clone(),
        ));

        let radio_service = RadioService::new(
            playlist_service.clone(),
            track_repository.clone(),
            config.clone(),
            queue_notify,
        );

        let auth_service = Arc::new(AuthService::new(cache.clone(), users_repository.clone()));

        let services = Services {
            sign_up_service,
            restore_service,
            auth_service,
            track_service,
            playlist_service,
            radio_service,
        };

        AppState {
            db_pool,
            config,
            services,
        }
    }
}

