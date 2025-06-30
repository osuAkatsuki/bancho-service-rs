use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct IsOnlineArgs {
    pub id: i64,
}

#[derive(Serialize)]
pub struct IsOnlineResponse {
    pub message: &'static str,
    pub result: bool,
    pub status: u16,
}

#[derive(Serialize)]
pub struct OnlineUsersResponse {}
