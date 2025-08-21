use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Beatmap {
    pub beatmap_id: i32,
    pub beatmapset_id: i32,
    pub beatmap_md5: String,
    pub song_name: String,
    pub file_name: String,
    pub ar: f32,
    pub od: f32,
    pub mode: i32,
    pub max_combo: i32,
    pub hit_length: i32,
    pub bpm: i32,
    pub ranked: i8,
    pub latest_update: i32,
    pub ranked_status_freezed: bool,
    pub playcount: i32,
    pub passcount: i32,
    #[sqlx(default)]
    pub rankedby: Option<i32>,
    pub rating: f64,
    #[sqlx(default)]
    pub bancho_ranked_status: Option<i16>,
}
