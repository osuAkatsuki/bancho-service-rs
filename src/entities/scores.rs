#[derive(sqlx::FromRow)]
pub struct MinimalScore {
    #[sqlx(rename = "id")]
    pub score_id: i64,
    #[sqlx(rename = "userid")]
    pub user_id: i64,
    #[sqlx(rename = "play_mode")]
    pub mode: i8,
    pub score: i64,
    #[sqlx(rename = "pp")]
    pub performance: f32,
    pub time: i32,

    pub beatmap_md5: String,
}

#[derive(sqlx::FromRow)]
pub struct LastUserScore {
    #[sqlx(rename = "id")]
    pub score_id: i64,
    #[sqlx(rename = "userid")]
    pub user_id: i64,
    #[sqlx(rename = "play_mode")]
    pub mode: i8,
    pub mods: i32,
    pub score: i64,
    #[sqlx(rename = "pp")]
    pub performance: f32,
    pub accuracy: f32,
    pub time: i32,

    pub beatmap_id: i32,
    #[sqlx(rename = "beatmapset_id")]
    pub beatmap_set_id: i32,
    pub beatmap_md5: String,
    pub song_name: String,
}

#[derive(sqlx::FromRow)]
pub struct FirstPlaceScore {
    #[sqlx(rename = "scoreid")]
    pub score_id: i64,
    pub beatmap_md5: String,
    pub mode: i8,
    pub rx: i8,
}

#[derive(sqlx::FromRow)]
pub struct NewFirstPlace {
    #[sqlx(rename = "id")]
    pub score_id: i64,
    #[sqlx(rename = "userid")]
    pub user_id: i64,
}
