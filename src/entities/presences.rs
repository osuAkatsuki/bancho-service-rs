use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Presence {
    pub user_id: i64,
    pub action: u8,
    pub info_text: String,
    pub beatmap_md5: String,
    pub beatmap_id: i32,
    pub mods: u32,
    pub mode: u8,
    pub ranked_score: u64,
    pub total_score: u64,
    pub accuracy: f64,
    pub playcount: u32,
    pub performance: u32,
    pub global_rank: usize,

    pub country_code: String,
    pub longitude: f32,
    pub latitude: f32,
    pub utc_offset: i8,
}
