use std::sync::Arc;

use crate::{
    error::app_error::{AppResult, ErrorCode},
    infrastucture::database::{
        models::{NewUser, User},
        pool::DbPool,
    },
};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::error::app_error::AppError;

pub struct UsersRepository {
    db_pool: Arc<DbPool>,
}

impl UsersRepository {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        UsersRepository { db_pool }
    }
    pub async fn create_user(&self, new_user: &NewUser) -> AppResult<User> {
        use crate::schema::users::dsl::*;
        let mut conn = self.db_pool.get().await?;
        let user = diesel::insert_into(users)
            .values(new_user)
            .get_result::<User>(&mut conn)
            .await;

        let user = match user {
            Ok(user) => user,
            Err(e) => {
                return Err(match e {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        _,
                    ) => AppError::Validation(
                        "Пользователь с таким именем или электронной почтой уже существует"
                            .to_string(),
                        Some(ErrorCode::UserAlreadyExists),
                    ),
                    other => AppError::Database(format!("Failed to create user: {}", other), None),
                });
            }
        };

        Ok(user)
    }

    pub async fn get_user_by_username(&self, user_name: &str) -> AppResult<User> {
        use crate::schema::users::dsl::*;
        let mut conn = self.db_pool.get().await?;
        let user = users
            .filter(username.eq(user_name))
            .first::<User>(&mut conn)
            .await;

        let user = match user {
            Ok(user) => user,
            Err(e) => {
                return Err(match e {
                    diesel::result::Error::NotFound => {
                        AppError::NotFound("User not found".to_string(), None)
                    }
                    other => {
                        AppError::Database(format!("Failed to retrieve user: {}", other), None)
                    }
                });
            }
        };

        Ok(user)
    }

    pub async fn get_user_by_email(&self, user_email: &str) -> AppResult<User> {
        use crate::schema::users::dsl::*;
        let mut conn = self.db_pool.get().await?;
        let user = users
            .filter(email.eq(user_email))
            .first::<User>(&mut conn)
            .await;

        let user = match user {
            Ok(user) => user,
            Err(e) => {
                return Err(match e {
                    diesel::result::Error::NotFound => {
                        AppError::NotFound("User not found".to_string(), None)
                    }
                    other => {
                        AppError::Database(format!("Failed to retrieve user: {}", other), None)
                    }
                });
            }
        };

        Ok(user)
    }
}
