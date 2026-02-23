pub enum AppCacheKey<'a> {
    SESSION(&'a str),
    SIGN_UP_OTP(&'a str),
    PLAYLIST(),
}

impl<'a> AppCacheKey<'a> {
    pub fn build_key(&self) -> String {
        match self {
            AppCacheKey::SESSION(session_id) => format!("AUTH_SESSION_{}", session_id),
            AppCacheKey::SIGN_UP_OTP(email) => format!("SIGN_UP_OTP_{}", email),
            AppCacheKey::PLAYLIST() => "PLAYLIST".to_string(),
        }
    }
}
