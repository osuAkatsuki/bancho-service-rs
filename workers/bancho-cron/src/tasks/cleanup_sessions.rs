use bancho_service::common::context::Context;
use bancho_service::common::error::ServiceResult;
use bancho_service::usecases::sessions;
use chrono::{TimeDelta, Utc};
use std::ops::Add;
use tracing::info;

const CLIENT_TIMEOUT: i64 = 5 * 60;

pub async fn cleanup_sessions<C: Context>(ctx: &C) -> ServiceResult<()> {
    let active_sessions = sessions::fetch_all(ctx).await?;
    let now = Utc::now();
    let timed_out = active_sessions
        .into_iter()
        .filter(|session| session.updated_at.add(TimeDelta::seconds(CLIENT_TIMEOUT)) < now);
    for session in timed_out {
        info!(
            session_id = session.session_id.to_string(),
            user_id = session.user_id,
            "Session timed out..."
        );
        sessions::delete(ctx, &session).await?;
    }
    Ok(())
}
