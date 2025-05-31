use chrono::{DateTime, Utc};

pub enum MatchEventType {
    MatchCreated,
    MatchDisbanded,
    MatchUserJoined,
    MatchUserLeft,
    MatchHostAssignment,
    MatchGamePlaythrough,
}

impl MatchEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchEventType::MatchCreated => "MATCH_CREATION",
            MatchEventType::MatchDisbanded => "MATCH_DISBAND",
            MatchEventType::MatchUserJoined => "MATCH_USER_JOIN",
            MatchEventType::MatchUserLeft => "MATCH_USER_LEFT",
            MatchEventType::MatchHostAssignment => "MATCH_HOST_ASSIGNMENT",
            MatchEventType::MatchGamePlaythrough => "MATCH_GAME_PLAYTHOUGH",
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct MatchEvent {
    pub id: i64,
    pub match_id: i64,
    pub game_id: Option<i64>,
    pub user_id: Option<i64>,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
}
