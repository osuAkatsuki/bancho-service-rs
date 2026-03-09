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
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct CreateSessionArgs {
    pub user_id: i64,
    pub username: String,
    pub privileges: i32,
    pub private_dms: bool,
    pub silence_end: Option<chrono::DateTime<chrono::Utc>>,
    pub ip_address: IpAddr,
}
