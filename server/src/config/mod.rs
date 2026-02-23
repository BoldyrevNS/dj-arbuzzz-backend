mod database;
mod redis;
mod secret;
mod smtp;
mod songs;

#[derive(Clone, Debug)]
pub enum AppEnvironment {
    Development,
    Production,
}

pub struct AppConfig {
    pub env: AppEnvironment,
    pub db_config: database::DatabaseConfig,
    pub redis_config: redis::RedisConfig,
    pub app_port: u16,
    pub music_api_url: String,
    pub music_api_token: String,
    pub smtp_config: smtp::SMTPConfig,
    pub secret_config: secret::SecretConfig,
    pub songs_config: songs::SongsConfig,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            env: Self::get_env(),
            db_config: database::DatabaseConfig::new(),
            redis_config: redis::RedisConfig::new(),
            app_port: Self::get_app_port(),
            music_api_url: Self::get_music_api_url(),
            music_api_token: Self::get_music_api_token(),
            smtp_config: smtp::SMTPConfig::new(),
            secret_config: secret::SecretConfig::new(),
            songs_config: songs::SongsConfig::new(),
        }
    }

    fn get_env() -> AppEnvironment {
        match std::env::var("ENV") {
            Ok(env) if env == "production" => AppEnvironment::Production,
            _ => AppEnvironment::Development,
        }
    }

    fn get_app_port() -> u16 {
        std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("APP_PORT must be a valid u16")
    }

    fn get_music_api_url() -> String {
        std::env::var("MUSIC_API_URL").expect("MUSIC_API_URL must be set")
    }

    fn get_music_api_token() -> String {
        std::env::var("MUSIC_API_TOKEN").expect("MUSIC_API_TOKEN must be set")
    }
}
