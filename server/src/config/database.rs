pub struct DatabaseConfig {
    pub url: String,
}

impl DatabaseConfig {
    pub fn new() -> Self {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        DatabaseConfig { url }
    }
}
