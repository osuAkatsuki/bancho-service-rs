use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct LastNowPlayingState {
    pub beatmap_id: i32,
    pub beatmap_set_id: i32,
    pub beatmap_md5: String,
    pub mode: u8,
    pub mods: u32,
}
