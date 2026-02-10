use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    dto::{
        request::auth::sign_up::{
            ResendOTPRequest, SignUpCompleteRequest, SignUpStartRequest, VerifyOTPRequest,
        },
        response::auth::sign_up::{ResendOTPResponse, SignUpStartResponse},
    },
    error::app_error::{AppError, ErrorCode},
    infrastucture::{
        cache::client::Cache, database::models::NewUser,
        repositories::users_repository::UsersRepository,
    },
};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};

use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::{
    error::app_error::AppResult,
    service::{otp_service::OTPService, smtp_service::SMTPService},
};
use redis::AsyncCommands;

#[derive(Serialize, Deserialize)]
struct SignUpOtpParams {
    otp_value: String,
    send_timestamp_seconds: u64,
    hash: String,
    verified: bool,
}

enum CacheKey<'a> {
    SignUpOTP(&'a str),
}

impl CacheKey<'_> {
    fn to_string(&self) -> String {
        match self {
            CacheKey::SignUpOTP(value) => format!("sign_up_otp:{}", value),
        }
    }
}

pub struct SignUpService {
    cache: Arc<Cache>,
    otp_service: Arc<OTPService>,
    smtp_service: Arc<SMTPService>,
    users_repository: Arc<UsersRepository>,
}

impl SignUpService {
    pub fn new(
        cache: Arc<Cache>,
        otp_service: Arc<OTPService>,
        smtp_service: Arc<SMTPService>,
        users_repository: Arc<UsersRepository>,
    ) -> Self {
        SignUpService {
            cache,
            otp_service,
            smtp_service,
            users_repository,
        }
    }

    pub async fn start_sign_up(
        &self,
        payload: SignUpStartRequest,
    ) -> AppResult<SignUpStartResponse> {
        match self
            .users_repository
            .get_user_by_email(&payload.email)
            .await
        {
            Ok(_) => {
                return Err(AppError::BadRequest(
                    "Пользователь с таким email уже зарегистрирован".to_string(),
                    Some(ErrorCode::UserAlreadyExists),
                ));
            }
            Err(_) => (),
        }

        match self.get_cached_sign_up_data(&payload.email).await {
            Ok(data) => {
                let current_timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if current_timestamp - data.send_timestamp_seconds < 60 {
                    return Ok(SignUpStartResponse {
                        hash: data.hash,
                        timeout_seconds: Some(
                            60 - (current_timestamp - data.send_timestamp_seconds) as u16,
                        ),
                    });
                }
            }
            Err(_) => {}
        };

        let otp = self.otp_service.generate(6)?;
        let hash = self.otp_service.make_otp_hash(&payload.email, otp);

        self.smtp_service
            .send_registration_otp(&payload.email, otp)
            .await?;

        self.cache_sign_up_data(
            &payload.email,
            &SignUpOtpParams {
                otp_value: otp.to_string(),
                send_timestamp_seconds: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                verified: false,
                hash: hash.clone(),
            },
        )
        .await?;

        Ok(SignUpStartResponse {
            timeout_seconds: None,
            hash,
        })
    }

    pub async fn verify_otp(&self, payload: VerifyOTPRequest) -> AppResult<()> {
        let cached_data = match self.get_cached_sign_up_data(&payload.email).await {
            Ok(data) => data,
            Err(_) => {
                return Err(crate::error::app_error::AppError::Unauthorized(
                    "Время жизни OTP истекло".to_string(),
                    Some(ErrorCode::OTPExpired),
                ));
            }
        };

        if self
            .otp_service
            .is_otp_expired(cached_data.send_timestamp_seconds)
        {
            return Err(crate::error::app_error::AppError::Unauthorized(
                "Время жизни OTP истекло".to_string(),
                Some(ErrorCode::OTPExpired),
            ));
        }

        println!(
            "Cached OTP: {}, Provided OTP: {}",
            cached_data.hash, payload.hash
        );

        if cached_data.hash != payload.hash {
            return Err(crate::error::app_error::AppError::Unauthorized(
                "Ошибка при проверке OTP".to_string(),
                Some(ErrorCode::WrongOTPHash),
            ));
        }

        if cached_data.otp_value != payload.otp {
            return Err(crate::error::app_error::AppError::Unauthorized(
                "Неверный OTP".to_string(),
                Some(ErrorCode::WrongOTP),
            ));
        }

        self.verify_cached_otp(&payload.email).await?;

        Ok(())
    }

    pub async fn resend_otp(&self, payload: ResendOTPRequest) -> AppResult<ResendOTPResponse> {
        let cached_data = match self.get_cached_sign_up_data(&payload.email).await {
            Ok(data) => data,
            Err(_) => {
                return Err(AppError::Unauthorized(
                    "Время жизни OTP истекло".to_string(),
                    Some(ErrorCode::OTPExpired),
                ));
            }
        };

        if self
            .otp_service
            .is_otp_expired(cached_data.send_timestamp_seconds)
        {
            return Err(AppError::Unauthorized(
                "Время жизни OTP истекло".to_string(),
                Some(ErrorCode::OTPExpired),
            ));
        }

        if cached_data.hash != payload.hash {
            return Err(crate::error::app_error::AppError::Unauthorized(
                "Ошибка при проверке данных для повторной отправки OTP".to_string(),
                Some(ErrorCode::OTPResendFailed),
            ));
        }

        let otp = self.otp_service.generate(6)?;
        let hash = self.otp_service.make_otp_hash(&payload.email, otp);

        self.smtp_service
            .send_registration_otp(&payload.email, otp)
            .await?;

        self.cache_sign_up_data(
            &payload.email,
            &SignUpOtpParams {
                otp_value: otp.to_string(),
                send_timestamp_seconds: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                verified: false,
                hash: hash.clone(),
            },
        )
        .await?;

        Ok(ResendOTPResponse {
            hash,
            timeout_seconds: 60,
        })
    }

    pub async fn sign_up_complete(&self, payload: SignUpCompleteRequest) -> AppResult<()> {
        let cached_data = self.get_cached_sign_up_data(&payload.email).await?;

        if cached_data.hash != payload.hash {
            return Err(crate::error::app_error::AppError::Unauthorized(
                "Ошибка при проверке OTP".to_string(),
                Some(ErrorCode::WrongOTPHash),
            ));
        }

        if self
            .otp_service
            .is_otp_expired(cached_data.send_timestamp_seconds)
        {
            return Err(crate::error::app_error::AppError::Unauthorized(
                "OTP expired".to_string(),
                Some(ErrorCode::OTPExpired),
            ));
        }

        let mut rng = OsRng;
        let salt = SaltString::generate(&mut rng);
        let hashed_password = Argon2::default()
            .hash_password(payload.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        self.users_repository
            .create_user(&NewUser {
                email: payload.email,
                username: payload.username,
                password: hashed_password,
            })
            .await?;
        Ok(())
    }

    async fn cache_sign_up_data(&self, email: &str, data: &SignUpOtpParams) -> AppResult<()> {
        let mut con = self.cache.get_async_conn().await?;
        let key = CacheKey::SignUpOTP(email).to_string();
        let expire_seconds = 7 * 60;
        let _: () = con
            .hset_multiple(
                &key,
                &[
                    ("otp_value", data.otp_value.to_string()),
                    (
                        "send_timestamp_seconds",
                        data.send_timestamp_seconds.to_string(),
                    ),
                    (
                        "verified",
                        if data.verified { "1" } else { "0" }.to_string(),
                    ),
                    ("hash", data.hash.clone()),
                ],
            )
            .await?;
        let _: () = con.expire(&key, expire_seconds).await?;
        Ok(())
    }

    async fn get_cached_sign_up_data(&self, email: &str) -> AppResult<SignUpOtpParams> {
        let mut con = self.cache.get_async_conn().await?;
        let key = CacheKey::SignUpOTP(email).to_string();
        let otp_value: String = con.hget(&key, "otp_value").await?;
        let send_timestamp_seconds: u64 = con.hget(&key, "send_timestamp_seconds").await?;
        let hash: String = con.hget(&key, "hash").await?;
        let verified: u8 = con.hget(&key, "verified").await?;

        Ok(SignUpOtpParams {
            otp_value,
            send_timestamp_seconds,
            verified: verified == 1,
            hash,
        })
    }

    async fn verify_cached_otp(&self, email: &str) -> AppResult<()> {
        let mut con = self.cache.get_async_conn().await?;
        let key = CacheKey::SignUpOTP(email).to_string();
        let _: () = con.hset(&key, "verified", 1).await?;

        Ok(())
    }
}
