use std::sync::Arc;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, encode, get_current_timestamp};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    config::AppConfig,
    error::app_error::{AppError, AppResult, ErrorCode},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    Access,
    Refresh,
    SignUp,
    Restore,
}

pub trait Token {
    fn exp(&self) -> u64;
    fn token_type(&self) -> TokenType;
}

pub struct TokenService {
    config: Arc<AppConfig>,
}

impl TokenService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        TokenService { config }
    }

    pub fn create_jwt<T: Serialize + Token>(&self, data: T) -> AppResult<String> {
        let secret = self.get_secret_by_token_type(&data.token_type());
        let token = encode(
            &Header::default(),
            &data,
            &EncodingKey::from_secret(secret.as_bytes()),
        )?;
        Ok(token)
    }

    pub fn get_claims_from_jwt<T: DeserializeOwned + Token>(
        &self,
        token: &str,
        token_type: TokenType,
    ) -> AppResult<T> {
        let secret = self.get_secret_by_token_type(&token_type);
        let token_data = jsonwebtoken::decode::<T>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )?;
        if get_current_timestamp() > token_data.claims.exp() {
            return Err(AppError::Unauthorized(
                "Токен устарел".to_string(),
                Some(ErrorCode::JWTExpired),
            ));
        }
        Ok(token_data.claims)
    }

    pub fn is_token_expired<T: Token + DeserializeOwned>(
        &self,
        token: &str,
        token_type: TokenType,
    ) -> AppResult<bool> {
        let claims: T = self.get_claims_from_jwt(token, token_type)?;
        Ok(get_current_timestamp() > claims.exp())
    }

    fn get_secret_by_token_type(&self, token_type: &TokenType) -> &str {
        match token_type {
            TokenType::Access => &self.config.secret_config.access_secret,
            TokenType::Refresh => &self.config.secret_config.refresh_secret,
            TokenType::SignUp => &self.config.secret_config.sign_up_secret,
            TokenType::Restore => &self.config.secret_config.restore_secret,
        }
    }
}
