use bancho_protocol::structures::SlotStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow)]
pub struct PersistentMatch {
    pub match_id: i64,
    pub name: String,
    pub private: bool,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct MultiplayerMatch {
    pub match_id: i64,
    pub name: String,
    pub password: String,
    pub in_progress: bool,
    pub powerplay: bool,
    pub mods: u32,
    pub beatmap_name: String,
    pub beatmap_md5: String,
    pub beatmap_id: i32,
    pub host_user_id: i64,
    pub mode: u8,
    pub win_condition: u8,
    pub team_type: u8,
    pub freemod_enabled: bool,
    pub random_seed: i32,
    pub last_game_id: Option<i64>,
}

#[derive(Copy, Clone, Default, Deserialize, Serialize)]
pub struct MultiplayerMatchSlot {
    pub status: u8,
    pub team: u8,
    pub mods: u32,
    pub user_id: Option<i64>,
    pub loaded: bool,
    pub skipped: bool,
    pub failed: bool,
    pub completed: bool,
}

impl MultiplayerMatchSlot {
    pub fn prepare(&mut self, user_id: i64) {
        self.status = SlotStatus::NotReady.bits();
        self.team = 0;
        self.mods = 0;
        self.user_id = Some(user_id);
        self.loaded = false;
        self.skipped = false;
        self.failed = false;
        self.completed = false;
    }
    pub fn clear(&mut self) {
        self.status = SlotStatus::Empty.bits();
        self.team = 0;
        self.mods = 0;
        self.user_id = None;
        self.loaded = false;
        self.skipped = false;
        self.failed = false;
        self.completed = false;
    }
}
