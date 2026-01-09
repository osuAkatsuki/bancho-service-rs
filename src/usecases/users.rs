use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::models::privileges::Privileges;
use crate::models::users::{User, VerifiedStatus};
use crate::repositories::streams::StreamName;
use crate::repositories::users;
use crate::usecases::{messages, sessions, streams};
use bancho_protocol::messages::server::{SilenceEnd, UserSilenced};
use chrono::Utc;

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

pub async fn fetch_one_by_username_safe<C: Context>(
    ctx: &C,
    username: &str,
) -> ServiceResult<User> {
    match users::fetch_one_by_username_safe(ctx, username).await {
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

pub async fn change_username<C: Context>(
    ctx: &C,
    user_id: i64,
    new_username: &str,
) -> ServiceResult<()> {
    match users::change_username(ctx, user_id, new_username).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn ban_user<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<()> {
    match users::ban(ctx, user_id).await {
        Ok(_) => {
            users::publish_ban_event(ctx, user_id).await?;
            Ok(())
        }
        Err(e) => unexpected(e),
    }
}

pub async fn unban_user<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<()> {
    match users::unban(ctx, user_id).await {
        Ok(_) => {
            users::publish_unban_event(ctx, user_id).await?;
            Ok(())
        }
        Err(e) => unexpected(e),
    }
}

pub async fn restrict_user<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<()> {
    match users::restrict(ctx, user_id).await {
        Ok(_) => {
            users::publish_ban_event(ctx, user_id).await?;
            Ok(())
        }
        Err(e) => unexpected(e),
    }
}

pub async fn unrestrict_user<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<()> {
    match users::unrestrict(ctx, user_id).await {
        Ok(_) => {
            users::publish_unban_event(ctx, user_id).await?;
            Ok(())
        }
        Err(e) => unexpected(e),
    }
}

pub async fn freeze_user<C: Context>(ctx: &C, user_id: i64, reason: &str) -> ServiceResult<()> {
    match users::freeze(ctx, user_id, reason).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn unfreeze_user<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<()> {
    match users::unfreeze(ctx, user_id).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn update_user_privileges<C: Context>(
    ctx: &C,
    user_id: i64,
    privileges: Privileges,
) -> ServiceResult<()> {
    match users::update_privileges(ctx, user_id, privileges).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn update_user_whitelist<C: Context>(
    ctx: &C,
    user_id: i64,
    whitelist_bit: i32,
) -> ServiceResult<()> {
    match users::update_whitelist(ctx, user_id, whitelist_bit).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn queue_username_change<C: Context>(
    ctx: &C,
    user_id: i64,
    new_username: &str,
) -> ServiceResult<()> {
    match users::queue_username_change(ctx, user_id, new_username).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn update_donor_expiry<C: Context>(
    ctx: &C,
    user_id: i64,
    donor_expire: i64,
) -> ServiceResult<()> {
    match users::update_donor_expiry(ctx, user_id, donor_expire).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_previous_overwrite<C: Context>(
    ctx: &C,
    user_id: i64,
) -> ServiceResult<Option<i64>> {
    match users::fetch_previous_overwrite(ctx, user_id).await {
        Ok(previous_overwrite) => Ok(previous_overwrite),
        Err(e) => unexpected(e),
    }
}

pub async fn unlock_overwrite<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<()> {
    match users::update_previous_overwrite(ctx, user_id, 1).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn update_previous_overwrite<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<()> {
    let now_timestamp = Utc::now().timestamp();
    match users::update_previous_overwrite(ctx, user_id, now_timestamp).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}
