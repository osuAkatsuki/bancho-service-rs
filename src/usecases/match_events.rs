use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::entities::match_events::MatchEventType;
use crate::repositories::match_events;

pub async fn create<C: Context>(
    ctx: &C,
    match_id: i64,
    event_type: MatchEventType,
    user_id: Option<i64>,
    game_id: Option<i64>,
) -> ServiceResult<()> {
    match match_events::create(ctx, match_id, event_type, user_id, game_id).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}
