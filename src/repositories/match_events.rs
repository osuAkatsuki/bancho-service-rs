use crate::common::context::Context;
use crate::entities::match_events::{MatchEvent, MatchEventType};
use chrono::Utc;

const TABLE_NAME: &str = "match_events";

pub async fn create<C: Context>(
    ctx: &C,
    match_id: i64,
    event_type: MatchEventType,
    user_id: Option<i64>,
    game_id: Option<i64>,
) -> sqlx::Result<MatchEvent> {
    let mut match_event = MatchEvent {
        id: 0,
        match_id,
        game_id,
        user_id,
        event_type: event_type.as_str().to_owned(),
        timestamp: Utc::now(),
    };
    const QUERY: &str = const_str::concat!(
        "INSERT INTO ",
        TABLE_NAME,
        " (match_id, game_id, user_id, event_type) VALUES (?, ?, ?, ?)"
    );
    let query_result = sqlx::query(QUERY)
        .bind(match_id)
        .bind(game_id)
        .bind(user_id)
        .bind(event_type.as_str())
        .execute(ctx.db())
        .await?;
    match_event.id = query_result.last_insert_id() as _;
    Ok(match_event)
}
