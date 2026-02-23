use crate::error::app_error::AppResult;

#[derive(Debug, Clone)]
pub struct Cache {
    pub client: redis::Client,
}

impl Cache {
    pub fn new(url: &str) -> Self {
        let client = redis::Client::open(url).expect("Invalid Redis URL");
        Cache { client }
    }

    pub async fn get_async_conn(&self) -> AppResult<redis::aio::MultiplexedConnection> {
        let con = self.client.get_multiplexed_async_connection().await?;
        Ok(con)
    }
}
