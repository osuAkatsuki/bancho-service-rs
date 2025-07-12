use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct NowPlayingState {
    pub beatmap_id: i32,
    pub beatmap_set_id: i32,
    pub beatmap_md5: String,
    pub beatmap_song_name: String,
    pub beatmap_max_combo: i32,
    pub mode: u8,
    pub mods: u32,
}
