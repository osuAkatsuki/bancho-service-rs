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

#[derive(Serialize)]
pub struct ResponseBase {
    pub message: &'static str,
    pub status: u16,
}

impl Default for ResponseBase {
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
    pub base: ResponseBase,
    pub result: bool,
}

#[derive(Default, Serialize)]
pub struct OnlineUsersResponse {
    #[serde(flatten)]
    pub base: ResponseBase,
    pub result: u64,
}

#[derive(Default, Serialize)]
pub struct VerifiedStatusResponse {
    #[serde(flatten)]
    pub base: ResponseBase,
    pub result: i8,
}
