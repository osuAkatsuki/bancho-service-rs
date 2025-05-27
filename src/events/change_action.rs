use crate::api::RequestContext;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::presences::{Presence, PresenceAction, PresenceStats};
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{presences, stats, streams};
use bancho_protocol::messages::MessageArgs;
use bancho_protocol::messages::client::ChangeAction;
use bancho_protocol::serde::BinarySerialize;

pub async fn handle(
    ctx: &RequestContext,
    session: &Session,
    args: ChangeAction<'_>,
) -> EventResult {
    let mut presence = match presences::fetch_one(ctx, session.user_id).await {
        Ok(presence) => presence,
        Err(AppError::PresencesNotFound) => {
            let mut presence = Presence::default();
            presence.user_id = session.user_id;
            presence
        }
        Err(e) => return Err(e),
    };

    let action = PresenceAction::from(args.action);
    if action == presence.action {
        return Ok(None);
    }

    let refresh_stats = presence.action.mode != action.mode;
    presence.action = action;

    if refresh_stats {
        let stats = stats::fetch_one(ctx, session.user_id, presence.action.mode).await?;
        let global_rank =
            stats::fetch_global_rank(ctx, session.user_id, presence.action.mode).await?;
        presence.stats = PresenceStats::from(stats, global_rank);
    }

    tracing::info!("Presence updated: {:?}", presence);
    let presence = presences::update(ctx, presence).await?;
    let user_panel = presence.to_bancho().as_message().serialize();
    if session.is_publicly_visible() {
        streams::broadcast_data(ctx, StreamName::Main, &user_panel, None, None).await?;
        Ok(None)
    } else {
        Ok(Some(user_panel))
    }
}
