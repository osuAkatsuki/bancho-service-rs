use chrono::{DateTime, Utc};
use tracing::warn;

pub enum MessageStatus {
    Active,
    Deleted,
}

impl<T: AsRef<str>> From<T> for MessageStatus {
    fn from(value: T) -> Self {
        match value.as_ref() {
            "active" => Self::Active,
            "deleted" => Self::Deleted,
            v => {
                warn!("Encountered unknown MessageStatus {v:?}");
                Self::Deleted
            }
        }
    }
}

impl MessageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageStatus::Active => "active",
            MessageStatus::Deleted => "deleted",
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct Message {
    pub id: u64,
    pub sender_id: i64,
    pub sender_name: String,
    pub recipient_id: Option<i64>,
    pub recipient_channel: Option<String>,
    pub content: String,
    pub unread: bool,
    pub created_at: DateTime<Utc>,
    pub status: String,
}
