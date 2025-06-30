use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Copy, Clone, Default, Debug, Deserialize, Serialize)]
pub struct SessionIdentity {
    pub session_id: Uuid,
    pub user_id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: i64,
    pub username: String,
    pub privileges: i32,
    pub create_ip_address: IpAddr,
    pub private_dms: bool,
    pub silence_end: Option<chrono::DateTime<chrono::Utc>>,
    pub primary: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct CreateSessionArgs {
    pub user_id: i64,
    pub username: String,
    pub privileges: i32,
    pub private_dms: bool,
    pub silence_end: Option<chrono::DateTime<chrono::Utc>>,
    pub ip_address: IpAddr,
    pub primary: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FallbackSession {
    pub token_id: String,
    pub user_id: i64,
    pub username: String,
    pub privileges: i64,
    pub whitelist: u8,
    pub kicked: bool,
    pub login_time: f64,
    pub ping_time: f64,
    pub utc_offset: i8,
    pub tournament: bool,
    pub block_non_friends_dm: bool,
    pub spectating_token_id: Option<String>,
    pub spectating_user_id: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub ip: String,
    pub country: u8,
    pub away_message: Option<String>,
    pub match_id: Option<i64>,

    pub match_slot_id: Option<u8>,

    pub last_np: Option<FallbackLastNp>,
    pub silence_end_time: i64,
    pub protocol_version: i64,
    pub spam_rate: i64,

    // stats
    pub action_id: u8,
    pub action_text: String,
    pub action_md5: String,
    pub action_mods: i64,
    pub game_mode: u8,
    pub relax: bool,
    pub autopilot: bool,
    pub beatmap_id: i64,
    pub ranked_score: i64,
    pub accuracy: f32,
    pub playcount: i64,
    pub total_score: i64,
    pub global_rank: i64,
    pub pp: i64,

    pub amplitude_device_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FallbackLastNp {
    pub beatmap_id: i64,
    pub mods: i64,
    pub accuracy: f32,
}
