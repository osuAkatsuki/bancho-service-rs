use crate::api::RequestContext;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::presences::{Presence, PresenceStats};
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{presences, stats, streams};

pub async fn handle(ctx: &RequestContext, session: &Session) -> EventResult {
    let mut presence = match presences::fetch_one(ctx, session.user_id).await {
        Ok(presence) => presence,
        Err(AppError::PresencesNotFound) => {
            let mut presence = Presence::default();
            presence.user_id = session.user_id;
            presence
        }
        Err(e) => return Err(e),
    };
    let stats = stats::fetch_one(ctx, session.user_id, presence.action.mode).await?;
    let global_rank = stats::fetch_global_rank(ctx, session.user_id, presence.action.mode).await?;
    let new_stats = PresenceStats::from(stats, global_rank);
    // No update needed
    if presence.stats == new_stats {
        return Ok(None);
    }
    presence.stats = new_stats;

    let user_panel = presence.user_panel();
    if presence.is_publicly_visible() {
        streams::broadcast_data(ctx, StreamName::Main, &user_panel, None, None).await?;
        Ok(None)
    } else {
        Ok(Some(user_panel))
    }
}
