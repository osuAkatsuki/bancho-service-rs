use crate::common::error::ServiceResult;
use crate::common::state::AppState;
use crate::models::presences::PresenceStats;
use crate::repositories::streams::StreamName;
use crate::usecases::{presences, sessions, stats, streams, users};
use bancho_protocol::messages::server::UserStatsRef;
use redis::Msg;
use tracing::info;

pub async fn handle(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let user_id: i64 = msg.get_payload()?;
    info!(user_id, "Handling update stats event for user");

    let user = users::fetch_one(&ctx, user_id).await?;
    let mut presence = presences::fetch_one(&ctx, user.user_id).await?;

    let stats = stats::fetch_one(&ctx, user.user_id, presence.action.mode).await?;
    let global_rank = stats::fetch_global_rank(&ctx, user.user_id, presence.action.mode).await?;
    presence.stats = PresenceStats::from(stats, global_rank);
    let presence = presences::update(&ctx, presence).await?;

    let bancho_stats = presence.to_bancho_stats();
    if user.privileges.is_publicly_visible() {
        streams::broadcast_message(
            &ctx,
            StreamName::Main,
            UserStatsRef(&bancho_stats),
            None,
            None,
        )
        .await?;
    } else {
        let sessions = sessions::fetch_by_user_id(&ctx, user_id).await?;
        for session in sessions {
            streams::broadcast_message(
                &ctx,
                StreamName::User(session.session_id),
                UserStatsRef(&bancho_stats),
                None,
                None,
            )
            .await?;
        }
    }

    info!(user_id, "Successfully handled update stats event for user");
    Ok(())
}
