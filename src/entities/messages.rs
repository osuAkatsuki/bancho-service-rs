use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow)]
pub struct Message {
    pub id: u64,
    pub sender_id: i64,
    pub sender_name: String,
    pub recipient_id: Option<i64>,
    pub recipient_channel: Option<String>,
    pub content: String,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
