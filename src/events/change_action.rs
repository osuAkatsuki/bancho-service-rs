use crate::api::RequestContext;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::presences::{
    Presence, PresenceAction, PresenceLocationInformation, PresenceStats,
};
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{location, presences, stats, streams};
use bancho_protocol::messages::client::ChangeAction;
use bancho_protocol::structures::Country;

pub async fn handle(
    ctx: &RequestContext,
    session: &Session,
    args: ChangeAction<'_>,
) -> EventResult {
    let mut presence = match presences::fetch_one(ctx, session.user_id).await {
        Ok(presence) => presence,
        Err(AppError::PresencesNotFound) => {
            let location =
                location::get_location(session.create_ip_address, Country::Unknown, false).await;

            Presence {
                user_id: session.user_id,
                username: session.username.clone(),
                privileges: session.privileges.to_bancho(),
                action: PresenceAction::default(),
                stats: PresenceStats::default(),
                location: PresenceLocationInformation {
                    country: location.country,
                    longitude: location.longitude,
                    latitude: location.latitude,
                    utc_offset: 0,
                },
            }
        }
        Err(e) => return Err(e),
    };

    let action = PresenceAction::from(args.action);
    if action == presence.action {
        return Ok(None);
    }

    let refresh_stats = action.has_mode_changed(&presence.action);
    presence.action = action;

    if refresh_stats {
        let stats = stats::fetch_one(ctx, session.user_id, presence.action.mode).await?;
        let global_rank =
            stats::fetch_global_rank(ctx, session.user_id, presence.action.mode).await?;
        presence.stats = PresenceStats::from(stats, global_rank);
    }

    tracing::info!(
        user_id = presence.user_id,
        "Changed Action: {:?}",
        presence.action.action
    );
    let presence = presences::update(ctx, presence).await?;
    let user_panel = presence.user_panel();
    if session.is_publicly_visible() {
        streams::broadcast_data(ctx, StreamName::Main, &user_panel, None, None).await?;
        Ok(None)
    } else {
        Ok(Some(user_panel))
    }
}
