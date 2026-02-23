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
        cache::{client::Cache, keys::AppCacheKey},
        database::models::NewUser,
        repositories::users_repository::UsersRepository,
    },
    service::token_service::{Token, TokenService, TokenType},
};

use argon2::{Argon2, PasswordHasher, password_hash::SaltString};

use jsonwebtoken::get_current_timestamp;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::{
    error::app_error::AppResult,
    service::{otp_service::OTPService, smtp_service::SMTPService},
};
use redis::AsyncCommands;

const TOKEN_LIFETIME_SEC: u64 = 60 * 11;
const CACHE_LIFETIME_SEC: i64 = 60 * 10;
const RESEND_OTP_TIMEOUT_SEC: u64 = 60;

#[derive(Serialize, Deserialize)]
struct TokenData {
    email: String,
    exp: u64,
    token_type: TokenType,
    created_at: u64,
}

impl Token for TokenData {
    fn exp(&self) -> u64 {
        self.exp
    }

    fn token_type(&self) -> TokenType {
        self.token_type.clone()
    }
}

#[derive(Serialize, Deserialize)]
struct CachedSignUpOtpParams {
    otp_value: String,
    send_timestamp_seconds: u64,
    verified: bool,
    token: String,
}

pub struct SignUpService {
    cache: Arc<Cache>,
    otp_service: Arc<OTPService>,
    smtp_service: Arc<SMTPService>,
    users_repository: Arc<UsersRepository>,
    token_service: Arc<TokenService>,
}

impl SignUpService {
    pub fn new(
        cache: Arc<Cache>,
        otp_service: Arc<OTPService>,
        smtp_service: Arc<SMTPService>,
        users_repository: Arc<UsersRepository>,
        token_service: Arc<TokenService>,
    ) -> Self {
        SignUpService {
            cache,
            otp_service,
            smtp_service,
            users_repository,
            token_service,
        }
    }

    pub async fn start_sign_up(
        &self,
        payload: SignUpStartRequest,
    ) -> AppResult<SignUpStartResponse> {
        if self.is_user_exists_in_database(&payload.email).await? {
            return Err(AppError::BadRequest(
                format!("User with email {} already exists", &payload.email),
                Some(ErrorCode::UserAlreadyExists),
            ));
        }

        match self.get_cached_data(&payload.email).await {
            Ok(data) => {
                let token_data = self
                    .token_service
                    .get_claims_from_jwt::<TokenData>(&data.token, TokenType::SignUp)?;
                return Ok(SignUpStartResponse {
                    token: data.token,
                    timeout_seconds: (token_data.exp - get_current_timestamp()) as u16,
                });
            }
            Err(_) => (),
        }

        let token = self.send_otp(payload.email).await?;

        Ok(SignUpStartResponse {
            token,
            timeout_seconds: TOKEN_LIFETIME_SEC as u16,
        })
    }

    pub async fn verify_otp(&self, payload: VerifyOTPRequest) -> AppResult<()> {
        let token_data = self
            .token_service
            .get_claims_from_jwt::<TokenData>(&payload.token, TokenType::SignUp)?;
        let cached_data = match self.get_cached_data(&token_data.email).await {
            Ok(data) => data,
            Err(_) => {
                return Err(crate::error::app_error::AppError::Unauthorized(
                    "OTP expired".to_string(),
                    Some(ErrorCode::OTPExpired),
                ));
            }
        };

        self.compare_tokens(&payload.token, &cached_data.token)?;
        if cached_data.otp_value != payload.otp {
            return Err(crate::error::app_error::AppError::Unauthorized(
                "Неверный OTP".to_string(),
                Some(ErrorCode::WrongOTP),
            ));
        }

        self.verify_otp_and_update_cache(&token_data.email).await?;

        Ok(())
    }

    pub async fn resend_otp(&self, payload: ResendOTPRequest) -> AppResult<ResendOTPResponse> {
        let token_data = self
            .token_service
            .get_claims_from_jwt::<TokenData>(&payload.token, TokenType::SignUp)?;

        let cached_data = match self.get_cached_data(&token_data.email).await {
            Ok(data) => data,
            Err(_) => {
                return Err(AppError::Unauthorized(
                    "OTP expired".to_string(),
                    Some(ErrorCode::OTPExpired),
                ));
            }
        };

        self.compare_tokens(&payload.token, &cached_data.token)?;

        if token_data.created_at + RESEND_OTP_TIMEOUT_SEC > get_current_timestamp() {
            return Err(AppError::TooManyRequests(
                "OTP was sent recently. Please wait before requesting a new one.".to_string(),
                Some(ErrorCode::ResendOTPTooManyRequests),
            ));
        }

        let token = self.send_otp(token_data.email.clone()).await?;

        Ok(ResendOTPResponse {
            token,
            timeout_seconds: 60,
        })
    }

    pub async fn sign_up_complete(&self, payload: SignUpCompleteRequest) -> AppResult<()> {
        let token_data = self
            .token_service
            .get_claims_from_jwt::<TokenData>(&payload.token, TokenType::SignUp)?;

        let cached_data = match self.get_cached_data(&token_data.email).await {
            Ok(data) => data,
            Err(_) => {
                return Err(crate::error::app_error::AppError::Unauthorized(
                    "OTP expired".to_string(),
                    Some(ErrorCode::OTPExpired),
                ));
            }
        };

        if !cached_data.verified {
            self.clear_cache(&token_data.email).await?;
            return Err(crate::error::app_error::AppError::Unauthorized(
                "OTP not verified".to_string(),
                Some(ErrorCode::OTPNotVerified),
            ));
        }

        let mut rng = OsRng;
        let salt = SaltString::generate(&mut rng);
        let hashed_password = Argon2::default()
            .hash_password(payload.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        match self
            .users_repository
            .create_user(&NewUser {
                email: token_data.email.clone(),
                username: payload.username,
                password: hashed_password,
            })
            .await
        {
            Ok(_) => self.clear_cache(&token_data.email).await?,
            Err(e) => {
                self.clear_cache(&token_data.email).await?;
                return Err(e);
            }
        };
        Ok(())
    }

    async fn cache_sign_up_data(&self, email: &str, data: &CachedSignUpOtpParams) -> AppResult<()> {
        let mut con = self.cache.get_async_conn().await?;
        let key = AppCacheKey::SIGN_UP_OTP(email).build_key();
        let _: () = con
            .hset_multiple(
                &key,
                &[
                    ("otp_value", data.otp_value.to_string()),
                    (
                        "send_timestamp_seconds",
                        data.send_timestamp_seconds.to_string(),
                    ),
                    ("token", data.token.clone()),
                    ("verified", "false".to_string()),
                ],
            )
            .await?;
        let _: () = con.expire(&key, CACHE_LIFETIME_SEC).await?;
        Ok(())
    }

    async fn get_cached_data(&self, email: &str) -> AppResult<CachedSignUpOtpParams> {
        let mut con = self.cache.get_async_conn().await?;
        let key = AppCacheKey::SIGN_UP_OTP(email).build_key();
        let otp_value: String = con.hget(&key, "otp_value").await?;
        let send_timestamp_seconds: u64 = con.hget(&key, "send_timestamp_seconds").await?;
        let token: String = con.hget(&key, "token").await?;
        let verified_str: String = con.hget(&key, "verified").await?;
        let _verified = match verified_str.as_str() {
            "true" => true,
            "false" => false,
            _ => {
                return Err(AppError::Internal(anyhow::anyhow!(
                    "Invalid verified value in cache"
                )));
            }
        };

        Ok(CachedSignUpOtpParams {
            otp_value,
            send_timestamp_seconds,
            token,
            verified: _verified,
        })
    }

    async fn verify_otp_and_update_cache(&self, email: &str) -> AppResult<()> {
        let mut con = self.cache.get_async_conn().await?;
        let key = AppCacheKey::SIGN_UP_OTP(email).build_key();
        let _: () = con.hset(&key, "verified", "true").await?;
        Ok(())
    }

    async fn clear_cache(&self, email: &str) -> AppResult<()> {
        let mut con = self.cache.get_async_conn().await?;
        let key = AppCacheKey::SIGN_UP_OTP(email).build_key();
        match con.del::<_, ()>(key).await {
            Ok(_) => (),
            Err(_) => (),
        };
        Ok(())
    }

    async fn is_user_exists_in_database(&self, email: &str) -> AppResult<bool> {
        match self.users_repository.get_user_by_email(email).await {
            Ok(_) => Ok(true),
            Err(AppError::NotFound(_, _)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn send_otp(&self, email: String) -> AppResult<String> {
        let otp = self.otp_service.generate(6)?;
        let token = self.token_service.create_jwt(TokenData {
            email: email.clone(),
            exp: get_current_timestamp() + TOKEN_LIFETIME_SEC,
            token_type: TokenType::SignUp,
            created_at: get_current_timestamp(),
        })?;

        self.smtp_service
            .send_registration_otp(email.as_str(), otp)
            .await?;

        self.cache_sign_up_data(
            email.as_str(),
            &CachedSignUpOtpParams {
                otp_value: otp.to_string(),
                send_timestamp_seconds: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                token: token.clone(),
                verified: false,
            },
        )
        .await?;

        Ok(token)
    }

    fn compare_tokens(&self, token1: &str, token2: &str) -> AppResult<()> {
        if token1 != token2 {
            return Err(AppError::Unauthorized(
                "Ошибка при проверке OTP".to_string(),
                Some(ErrorCode::WrongOTPToken),
            ));
        }
        Ok(())
    }
}
