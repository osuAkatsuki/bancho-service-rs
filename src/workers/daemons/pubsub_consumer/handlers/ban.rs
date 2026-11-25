use crate::common::error::ServiceResult;
use crate::common::state::AppState;
use crate::usecases::{scores, sessions, stats, users};
use redis::Msg;
use tracing::{error, info};

pub async fn handle(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let user_id: i64 = msg.get_payload()?;
    info!(user_id, "Handling ban event for user");

    let user = users::fetch_one(&ctx, user_id).await?;
    if user.privileges.is_publicly_visible() && user.privileges.can_login() {
        return Ok(());
    }

    let (sessions, leaderboard, first_places) = tokio::join!(
        revoke_sessions(&ctx, user_id),
        stats::remove_from_all_leaderboards(&ctx, user_id, user.country),
        scores::remove_first_places(&ctx, user_id, None, None),
    );
    sessions?;
    leaderboard?;
    first_places?;

    info!(user_id, "Successfully handled ban event for user");
    Ok(())
}

async fn revoke_sessions(ctx: &AppState, user_id: i64) -> ServiceResult<()> {
    let mut result = Ok(());
    for session in sessions::fetch_by_user_id(ctx, user_id).await? {
        if let Err(err) = sessions::delete(ctx, &session).await {
            error!(user_id, session_id = %session.session_id, ?err, "Failed to tear down session");
            result = Err(err);
        }
    }
    result
}
