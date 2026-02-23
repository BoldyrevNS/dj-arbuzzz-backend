use std::sync::Arc;

use diesel_async::RunQueryDsl;

use crate::infrastucture::database::{
    models::{NewUserTrack, UserTrack},
    pool::DbPool,
};

pub struct UserTrackRepository {
    db_pool: Arc<DbPool>,
}

impl UserTrackRepository {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        UserTrackRepository { db_pool }
    }

    pub async fn create_user_track(
        &self,
        new_user_track: &NewUserTrack,
    ) -> crate::error::app_error::AppResult<UserTrack> {
        use crate::schema::user_tracks::dsl::*;
        let mut con = self.db_pool.get().await?;
        let user_track = diesel::insert_into(user_tracks)
            .values(new_user_track)
            .get_result::<UserTrack>(&mut con)
            .await?;
        Ok(user_track)
    }
}
