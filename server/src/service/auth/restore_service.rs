use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    dto::{
        request::auth::restore::StartRestoreRequest, response::auth::restore::StartRestoreResponse,
    },
    error::app_error::AppResult,
    infrastucture::{cache::client::Cache, repositories::users_repository::UsersRepository},
    service::{
        otp_service::OTPService,
        smtp_service::SMTPService,
        token_service::{Token, TokenService, TokenType},
    },
};

pub struct RestoreService {
    cache: Arc<Cache>,
    otp_service: Arc<OTPService>,
    smtp_service: Arc<SMTPService>,
    users_repository: Arc<UsersRepository>,
    token_service: Arc<TokenService>,
}

#[derive(Serialize, Deserialize)]
struct TokenData {
    email: String,
    exp: u64,
    token_type: TokenType,
}

impl Token for TokenData {
    fn exp(&self) -> u64 {
        self.exp
    }
    fn token_type(&self) -> TokenType {
        self.token_type.clone()
    }
}

impl RestoreService {
    pub fn new(
        cache: Arc<Cache>,
        otp_service: Arc<OTPService>,
        smtp_service: Arc<SMTPService>,
        users_repository: Arc<UsersRepository>,
        token_service: Arc<TokenService>,
    ) -> Self {
        RestoreService {
            cache,
            otp_service,
            smtp_service,
            users_repository,
            token_service,
        }
    }
}
