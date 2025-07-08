use bancho_protocol::structures::{Mode, Mods};

pub struct NowPlayingMessage {
    pub beatmap_id: i32,
    pub mode: Mode,
    pub mods: Mods,
}
