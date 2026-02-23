use std::sync::Arc;

use crate::{
    error::app_error::AppResult,
    infrastucture::database::{
        models::{NewTrack, NewUserTrack, Track, UserTrack},
        pool::DbPool,
    },
};

use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel::OptionalExtension;

pub struct TrackRepository {
    db_pool: Arc<DbPool>,
}

impl TrackRepository {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        TrackRepository { db_pool }
    }

    pub async fn create_track(&self, new_track: &NewTrack) -> AppResult<Track> {
        use crate::schema::tracks::dsl::*;
        let mut con = self.db_pool.get().await?;
        let track = diesel::insert_into(tracks)
            .values(new_track)
            .get_result::<Track>(&mut con)
            .await?;

        Ok(track)
    }

    pub async fn create_track_with_user_track(
        &self,
        new_track: &NewTrack,
        user_id_val: i32,
    ) -> AppResult<(Track, UserTrack)> {
        use crate::schema::tracks::dsl::*;
        use crate::schema::user_tracks::dsl::*;

        let mut conn = self.db_pool.get().await?;

        let result = conn
            .transaction::<(Track, UserTrack), diesel::result::Error, _>(|tx_conn| {
                let new_track_ref = new_track;
                Box::pin(async move {
                    let track = match tracks
                        .filter(owner_id.eq(new_track_ref.owner_id))
                        .filter(song_id.eq(new_track_ref.song_id))
                        .first::<Track>(tx_conn)
                        .await
                    {
                        Ok(found_track) => {
                            diesel::update(tracks)
                                .filter(owner_id.eq(new_track_ref.owner_id))
                                .filter(song_id.eq(new_track_ref.song_id))
                                .set(listens_count.eq(found_track.listens_count + 1))
                                .get_result::<Track>(tx_conn)
                                .await?
                        }
                        Err(diesel::result::Error::NotFound) => {
                            diesel::insert_into(tracks)
                                .values(new_track_ref)
                                .get_result::<Track>(tx_conn)
                                .await?
                        }
                        Err(e) => return Err(e),
                    };

                    let new_user_track = NewUserTrack {
                        user_id: user_id_val,
                        track_id: track.id,
                    };

                    let inserted_user_track = diesel::insert_into(user_tracks)
                        .values(&new_user_track)
                        .get_result::<UserTrack>(tx_conn)
                        .await?;

                    Ok((track, inserted_user_track))
                })
            })
            .await?;

        Ok(result)
    }

    pub async fn find_track_by_track_owner(
        &self,
        track_id_val: i32,
        owner_id_val: i32,
    ) -> AppResult<Track> {
        use crate::schema::tracks::dsl::*;
        let mut con = self.db_pool.get().await?;
        let track = tracks
            .filter(owner_id.eq(owner_id_val))
            .filter(song_id.eq(track_id_val))
            .first::<Track>(&mut con)
            .await?;

        Ok(track)
    }

    pub async fn find_random_track(&self) -> AppResult<Track> {
        use diesel::sql_query;
        let mut con = self.db_pool.get().await?;
        let track = sql_query(
            "SELECT id, song_id, owner_id, download_url, title, artist, \
             duration_sec, likes_count, listens_count \
             FROM tracks ORDER BY RANDOM() LIMIT 1",
        )
        .get_result::<Track>(&mut con)
        .await
        .optional()?
        .ok_or_else(|| {
            crate::error::app_error::AppError::NotFound(
                "No tracks in database".to_string(),
                None,
            )
        })?;
        Ok(track)
    }
}
