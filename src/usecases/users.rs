use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::models::users::{User, VerifiedStatus};
use crate::repositories::streams::StreamName;
use crate::repositories::users;
use crate::usecases::{messages, sessions, streams};
use bancho_protocol::messages::server::{SilenceEnd, UserSilenced};

const SILENCE_AUTO_DELETE_INTERVAL_SECONDS: u64 = 60;

pub async fn fetch_one<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<User> {
    match users::fetch_one(ctx, user_id).await {
        Ok(user) => User::try_from(user),
        Err(sqlx::Error::RowNotFound) => Err(AppError::UsersNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_one_by_username<C: Context>(ctx: &C, username: &str) -> ServiceResult<User> {
    match users::fetch_one_by_username(ctx, username).await {
        Ok(user) => User::try_from(user),
        Err(sqlx::Error::RowNotFound) => Err(AppError::UsersNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn silence_user<C: Context>(
    ctx: &C,
    user_id: i64,
    silence_reason: &str,
    silence_seconds: i64,
) -> ServiceResult<()> {
    users::silence_user(ctx, user_id, silence_reason, silence_seconds).await?;
    messages::delete_recent(ctx, user_id, SILENCE_AUTO_DELETE_INTERVAL_SECONDS).await?;

    let sessions = sessions::fetch_by_user_id(ctx, user_id).await?;
    for session in sessions {
        let session_id = session.session_id;
        sessions::silence(ctx, session, silence_seconds).await?;
        // Tell the user that they have been silenced
        let silence_end = SilenceEnd {
            seconds_left: silence_seconds as _,
        };
        streams::broadcast_message(ctx, StreamName::User(session_id), silence_end, None, None)
            .await?;
    }

    // Tell all other users that the user has been silenced
    let user_silenced = UserSilenced {
        user_id: user_id as _,
    };
    streams::broadcast_message(ctx, StreamName::Main, user_silenced, None, None).await?;
    Ok(())
}

pub async fn fetch_verified_status<C: Context>(
    ctx: &C,
    user_id: i64,
) -> ServiceResult<VerifiedStatus> {
    let user = fetch_one(ctx, user_id).await?;
    if user.privileges.is_pending_verification() {
        Ok(VerifiedStatus::PendingVerification)
    } else if !user.privileges.is_publicly_visible() {
        Ok(VerifiedStatus::Multiaccount)
    } else {
        Ok(VerifiedStatus::Verified)
    }
}
