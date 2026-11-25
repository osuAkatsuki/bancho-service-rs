use crate::api::RequestContext;
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::entities::gamemodes::Gamemode;
use crate::entities::sessions::CreateSessionArgs;
use crate::models::bancho::{ClientVersion, LoginArgs};
use crate::models::presences::Presence;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::models::users::User;
use crate::repositories::streams::StreamName;
use crate::repositories::{ip_logs, sessions, users};
use crate::usecases::{
    channels, hardware_logs, location, multiplayer, presences, spectators, stats, streams,
};
use bancho_protocol::messages::server::UserLogout;
use chrono::TimeDelta;
use uuid::Uuid;

pub const USER_SESSIONS_LIMIT: u64 = 20;
pub const TOURNAMENT_STAFF_SESSIONS_LIMIT: u64 = 40;

pub async fn create(ctx: &RequestContext, args: LoginArgs) -> ServiceResult<(Session, Presence)> {
    if args.client_info.client_version.is_outdated() {
        return Err(AppError::ClientTooOld);
    }

    let user = match users::fetch_one_by_username(ctx, &args.identifier).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(AppError::SessionsInvalidCredentials),
        Err(e) => return unexpected(e),
    };

    if !bcrypt::verify(&args.secret, &user.password_md5)
        .map_err(|_| AppError::SessionsInvalidCredentials)?
    {
        return Err(AppError::SessionsInvalidCredentials);
    }

    let mut user = User::try_from(user)?;
    if !user.privileges.contains(Privileges::CanLogin)
        && !(user.privileges.is_pending_verification() && user.ban_datetime == 0)
    {
        return Err(AppError::SessionsLoginForbidden);
    }

    // Until circles is released, only admins may log in with it
    if matches!(args.client_info.client_version, ClientVersion::Circles)
        && !user.privileges.is_admin()
    {
        return Err(AppError::UnsupportedClientVersion);
    }

    let ip_address = ctx.request_ip.ip_addr;
    let user_verification_pending = user.privileges.is_pending_verification();

    ip_logs::create(ctx, user.user_id, ip_address).await?;
    hardware_logs::create(
        ctx,
        user.user_id,
        user_verification_pending,
        &args.client_info.client_hashes,
    )
    .await?;

    hardware_logs::check_for_multiaccounts(
        ctx,
        user.user_id,
        &user.username,
        user_verification_pending,
        &args.client_info.client_hashes,
    )
    .await?;

    let user_session_count = sessions::fetch_user_session_count(ctx, user.user_id).await?;
    if (!user.privileges.is_tournament_staff() && user_session_count >= USER_SESSIONS_LIMIT)
        || user_session_count >= TOURNAMENT_STAFF_SESSIONS_LIMIT
    {
        return Err(AppError::SessionsLimitReached);
    }

    if user_verification_pending {
        users::verify_user(ctx, user.user_id).await?;
        user = User::try_from(users::fetch_one(ctx, user.user_id).await?)?;
        if !user.privileges.can_login() || user.privileges.is_pending_verification() {
            return Err(AppError::SessionsLoginForbidden);
        }
    }

    let stats = stats::fetch_one(ctx, user.user_id, Gamemode::Standard).await?;
    let rank = stats::fetch_global_rank(ctx, user.user_id, Gamemode::Standard).await?;

    let location_info =
        location::get_location(ip_address, user.country, args.client_info.display_city).await;
    let session = sessions::create(
        ctx,
        CreateSessionArgs {
            ip_address,
            user_id: user.user_id,
            username: user.username.clone(),
            privileges: user.privileges.bits(),
            silence_end: user.silence_end,
            private_dms: args.client_info.pm_private,
        },
    )
    .await?;
    // after creating the presence, the user becomes visible on the users panel
    let presence = presences::create_default(
        ctx,
        user.user_id,
        user.username,
        user.privileges,
        stats.ranked_score,
        stats.total_score,
        stats.avg_accuracy,
        stats.playcount,
        stats.pp,
        rank,
        location_info.country,
        location_info.latitude,
        location_info.longitude,
        args.client_info.utc_offset,
    )
    .await?;
    Ok((Session::from(session), presence))
}

pub async fn fetch_one<C: Context>(ctx: &C, session_id: Uuid) -> ServiceResult<Session> {
    match sessions::fetch_one(ctx, session_id).await {
        Ok(Some(session)) => {
            let session = Session::from(session);
            match session.is_expired() {
                false => Ok(session),
                true => {
                    delete(ctx, &session).await?;
                    Err(AppError::SessionsNotFound)
                }
            }
        }
        Ok(None) => Err(AppError::SessionsNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_all<C: Context>(ctx: &C) -> ServiceResult<impl Iterator<Item = Session>> {
    match sessions::fetch_all(ctx).await {
        Ok(sessions) => Ok(sessions.map(Session::from)),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_by_user_id<C: Context>(
    ctx: &C,
    user_id: i64,
) -> ServiceResult<impl Iterator<Item = Session>> {
    let sessions = sessions::fetch_by_user_id(ctx, user_id).await?;
    Ok(sessions.map(Session::from))
}

pub async fn fetch_by_username<C: Context>(
    ctx: &C,
    username: &str,
) -> ServiceResult<impl Iterator<Item = Session>> {
    let sessions = sessions::fetch_by_username(ctx, username).await?;
    Ok(sessions.map(Session::from))
}

pub async fn is_online<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<bool> {
    let is_online = sessions::is_online(ctx, user_id).await?;
    Ok(is_online)
}

pub async fn fetch_count<C: Context>(ctx: &C) -> ServiceResult<u64> {
    let online_count = sessions::fetch_count(ctx).await?;
    Ok(online_count)
}

pub async fn extend<C: Context>(ctx: &C, session_id: Uuid) -> ServiceResult<Session> {
    update(ctx, session_id, |_| {})
        .await?
        .ok_or(AppError::SessionsNotFound)
}

pub async fn update<C: Context>(
    ctx: &C,
    session_id: Uuid,
    change: impl Fn(&mut crate::entities::sessions::Session),
) -> ServiceResult<Option<Session>> {
    Ok(sessions::update(ctx, session_id, change)
        .await?
        .map(Session::from))
}

pub async fn delete<C: Context>(ctx: &C, session: &Session) -> ServiceResult<()> {
    channels::leave_all(ctx, session.session_id).await?;
    spectators::leave(ctx, session, None).await?;
    spectators::close(ctx, session.session_id).await?;
    multiplayer::leave(ctx, session.identity(), None).await?;

    let user_session_count = sessions::delete(ctx, session.session_id, session.user_id).await?;
    streams::clear_stream(ctx, StreamName::User(session.session_id)).await?;
    streams::leave_all(ctx, session.session_id).await?;

    if user_session_count == 0 {
        presences::delete(ctx, session.user_id).await?;
        // notify everyone
        let logout_notification = UserLogout::new(session.user_id as _);
        streams::broadcast_message(ctx, StreamName::Main, logout_notification, None, None).await?;
    }
    Ok(())
}

pub async fn set_private_dms<C: Context>(
    ctx: &C,
    session: &Session,
    private_dms: bool,
) -> ServiceResult<()> {
    update(ctx, session.session_id, |current| {
        current.private_dms = private_dms;
    })
    .await?;
    Ok(())
}

/// Silences the given session for the given amount of seconds.
pub async fn silence<C: Context>(
    ctx: &C,
    session: Session,
    silence_seconds: i64,
) -> ServiceResult<()> {
    let silence_end = chrono::Utc::now() + TimeDelta::seconds(silence_seconds);
    update(ctx, session.session_id, |current| {
        current.silence_end = Some(silence_end);
    })
    .await?;
    Ok(())
}
