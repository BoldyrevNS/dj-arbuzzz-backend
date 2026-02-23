// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;
}

diesel::table! {
    tracks (id) {
        id -> Int4,
        song_id -> Int4,
        owner_id -> Int4,
        download_url -> Varchar,
        title -> Varchar,
        artist -> Varchar,
        duration_sec -> Int4,
        likes_count -> Int4,
    }
}

diesel::table! {
    user_likes (id) {
        id -> Int4,
        user_id -> Int4,
        track_id -> Int4,
        liked_at -> Timestamp,
    }
}

diesel::table! {
    user_tracks (id) {
        id -> Int4,
        user_id -> Int4,
        track_id -> Int4,
        added_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserRole;

    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        email -> Varchar,
        role -> UserRole,
    }
}

diesel::joinable!(user_likes -> tracks (track_id));
diesel::joinable!(user_likes -> users (user_id));
diesel::joinable!(user_tracks -> tracks (track_id));
diesel::joinable!(user_tracks -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(tracks, user_likes, user_tracks, users,);
