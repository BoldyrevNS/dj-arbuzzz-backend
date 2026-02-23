use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(diesel_derive_enum::DbEnum, Debug, Clone, Serialize, Deserialize)]
#[db_enum(existing_type_path = "crate::schema::sql_types::UserRole")]
pub enum UserRole {
    USER,
    ADMIN,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, QueryableByName, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::tracks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Track {
    pub id: i32,
    pub song_id: i32,
    pub owner_id: i32,
    pub download_url: String,
    pub title: String,
    pub artist: String,
    pub duration_sec: i32,
    pub likes_count: i32,
    pub listens_count: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::tracks)]
pub struct NewTrack {
    pub song_id: i32,
    pub owner_id: i32,
    pub download_url: String,
    pub title: String,
    pub artist: String,
    pub duration_sec: i32,
    pub likes_count: Option<i32>,
    pub listens_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_tracks)]
pub struct UserTrack {
    pub id: i32,
    pub user_id: i32,
    pub track_id: i32,
    pub added_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::user_tracks)]
pub struct NewUserTrack {
    pub user_id: i32,
    pub track_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_likes)]
pub struct UserLike {
    user_id: i32,
    track_id: i32,
    liked_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::user_likes)]
pub struct NewUserLike {
    pub user_id: i32,
    pub track_id: i32,
}
