pub struct SongsConfig {
    pub songs_dir_path: String,
}

impl SongsConfig {
    pub fn new() -> Self {
        let songs_dir_path = std::env::var("SONGS_DIR_PATH").expect("SONGS_DIR_PATH must be set");
        SongsConfig { songs_dir_path }
    }
}
