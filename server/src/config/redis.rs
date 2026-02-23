pub struct RedisConfig {
    pub url: String,
}

impl RedisConfig {
    pub fn new() -> Self {
        let url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
        RedisConfig { url }
    }
}
