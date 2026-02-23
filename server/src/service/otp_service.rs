use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use rand::{Rng, thread_rng};

use crate::error::app_error::{AppError, AppResult};

#[derive(Hash)]
pub struct HashData {
    pub email: String,
    pub otp: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    email: String,
    otp: String,
    exp: u64,
}

pub struct OTPService {}

impl OTPService {
    pub fn new() -> Self {
        OTPService {}
    }

    pub fn generate(&self, len: u32) -> AppResult<u32> {
        if len < 1 {
            return Err(AppError::Internal(anyhow::anyhow!(
                "OTP length must be greater than 0"
            )));
        }
        if len > 9 {
            return Err(AppError::Internal(anyhow::anyhow!(
                "OTP length must be lesser than 10"
            )));
        }

        let lower = 10u32.pow(len - 1);
        let upper = 10u32.pow(len);
        Ok(thread_rng().gen_range(lower..upper))
    }

    pub fn make_otp_hash(&self, email: &str, otp: u32) -> String {
        let mut hasher = DefaultHasher::new();
        let data = HashData {
            email: email.to_string(),
            otp,
        };
        data.hash(&mut hasher);
        hasher.finish().to_string()
    }
    pub fn is_otp_expired(&self, send_timestamp_seconds: u64) -> bool {
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_timestamp - send_timestamp_seconds > 5 * 60
    }
}
