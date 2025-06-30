use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct IsOnlineArgs {
    #[serde(rename = "id")]
    pub user_id: i64,
}

#[derive(Deserialize)]
pub struct IsVerifiedArgs {
    #[serde(rename = "u")]
    pub user_id: i64,
}

#[derive(Deserialize)]
pub struct FetchPlayerMatchDetailsArgs {
    #[serde(rename = "id")]
    pub user_id: i64,
}

#[derive(Deserialize)]
pub struct SendChatbotMessageArgs {
    #[serde(rename = "k")]
    pub key: String,
    #[serde(rename = "to")]
    pub channel: String,
    #[serde(rename = "msg")]
    pub content: String,
}

#[derive(Serialize)]
pub struct BaseSuccessData {
    pub message: &'static str,
    pub status: u16,
}

impl Default for BaseSuccessData {
    fn default() -> Self {
        Self {
            message: "ok",
            status: 200,
        }
    }
}

#[derive(Default, Serialize)]
pub struct IsOnlineResponse {
    #[serde(flatten)]
    pub base: BaseSuccessData,
    pub result: bool,
}

#[derive(Default, Serialize)]
pub struct OnlineUsersResponse {
    #[serde(flatten)]
    pub base: BaseSuccessData,
    pub result: u64,
}

#[derive(Default, Serialize)]
pub struct VerifiedStatusResponse {
    #[serde(flatten)]
    pub base: BaseSuccessData,
    pub result: i8,
}

#[derive(Serialize)]
pub struct PlayerMatchDetails {
    pub match_name: String,
    pub match_id: i64,
    pub slot_id: u8,
    pub game_id: i64,
    pub team: u8,
}

#[derive(Default, Serialize)]
pub struct PlayerMatchDetailsResponse {
    #[serde(flatten)]
    pub base: BaseSuccessData,
    pub result: Option<PlayerMatchDetails>,
}
