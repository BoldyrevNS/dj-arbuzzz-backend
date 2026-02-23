use crate::error::app_error::AppResult;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;

pub type DbPool = Pool<diesel_async::AsyncPgConnection>;

pub async fn create_pool(db_url: &str) -> AppResult<DbPool> {
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let pool = Pool::builder(config).build()?;

    Ok(pool)
}
