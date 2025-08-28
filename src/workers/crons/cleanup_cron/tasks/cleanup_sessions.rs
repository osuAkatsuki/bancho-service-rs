use crate::common::context::Context;
use crate::common::error::ServiceResult;
use crate::usecases::sessions;
use chrono::{TimeDelta, Utc};
use std::ops::Add;
use tracing::{error, info};

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
        if let Err(e) = sessions::delete(ctx, &session).await {
            error!(
                session_id = session.session_id.to_string(),
                user_id = session.user_id,
                "Failed to time out session: {e:?}",
            );
        }
    }
    Ok(())
}
