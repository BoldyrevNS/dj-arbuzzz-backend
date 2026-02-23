pub struct SecretConfig {
    pub sign_up_secret: String,
    pub restore_secret: String,
    pub access_secret: String,
    pub refresh_secret: String,
}

impl SecretConfig {
    pub fn new() -> Self {
        let sign_up_secret = std::env::var("SIGN_UP_SECRET").expect("SIGN_UP_SECRET must be set");
        let restore_secret = std::env::var("RESTORE_SECRET").expect("RESTORE_SECRET must be set");
        let access_secret = std::env::var("ACCESS_SECRET").expect("ACCESS_SECRET must be set");
        let refresh_secret = std::env::var("REFRESH_SECRET").expect("REFRESH_SECRET must be set");
        SecretConfig {
            sign_up_secret,
            restore_secret,
            access_secret,
            refresh_secret,
        }
    }
}
