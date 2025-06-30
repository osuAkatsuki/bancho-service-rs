use crate::api::RequestContext;
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::entities::sessions::CreateSessionArgs;
use crate::models::Gamemode;
use crate::models::bancho::LoginArgs;
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

pub async fn create(ctx: &RequestContext, args: LoginArgs) -> ServiceResult<(Session, Presence)> {
    if args.client_info.osu_version.is_outdated() {
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
    if !user.privileges.contains(Privileges::CanLogin) {
        return Err(AppError::SessionsLoginForbidden);
    }

    let ip_address = ctx.request_ip.ip_addr;
    let user_verification_pending = user.privileges.contains(Privileges::PendingVerification);

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

    if user_verification_pending {
        users::verify_user(ctx, user.user_id).await?;
        user.privileges.remove(Privileges::PendingVerification);
    }

    let stats = stats::fetch_one(ctx, user.user_id, Gamemode::Standard).await?;
    let rank = stats::fetch_global_rank(ctx, user.user_id, Gamemode::Standard).await?;

    let location_info =
        location::get_location(ip_address, user.country, args.client_info.display_city).await;
    let mut sessions = fetch_by_user_id(ctx, user.user_id).await?;
    let already_logged_in = sessions.next().is_some();
    let session = sessions::create(
        ctx,
        CreateSessionArgs {
            ip_address,
            user_id: user.user_id,
            username: user.username.clone(),
            privileges: user.privileges.bits(),
            silence_end: user.silence_end,
            private_dms: args.client_info.pm_private,
            primary: !already_logged_in,
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
        Ok(Some(session)) if session.is_expired() => {
            sessions::delete(ctx, session_id, session.user_id, &session.username).await?;
            Err(AppError::SessionsNotFound)
        }
        Ok(Some(session)) => Ok(Session::from(session)),
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

pub async fn fetch_primary_by_user_id<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<Session> {
    let mut host_sessions = fetch_by_user_id(ctx, user_id).await?;
    host_sessions
        .find(|s| s.primary)
        .ok_or(AppError::SessionsNotFound)
}

pub async fn fetch_by_username<C: Context>(
    ctx: &C,
    username: &str,
) -> ServiceResult<impl Iterator<Item = Session>> {
    let sessions = sessions::fetch_by_username(ctx, username).await?;
    Ok(sessions.map(Session::from))
}

pub async fn fetch_primary_by_username<C: Context>(
    ctx: &C,
    username: &str,
) -> ServiceResult<Session> {
    let mut host_sessions = fetch_by_username(ctx, username).await?;
    host_sessions
        .find(|s| s.primary)
        .ok_or(AppError::SessionsNotFound)
}

pub async fn is_online<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<bool> {
    let is_online = sessions::is_online(ctx, user_id).await?;
    Ok(is_online)
}

pub async fn extend<C: Context>(ctx: &C, session_id: Uuid) -> ServiceResult<Session> {
    let session = fetch_one(ctx, session_id).await?;
    match sessions::extend(ctx, session.into()).await {
        Ok(session) => Ok(Session::from(session)),
        Err(e) => unexpected(e),
    }
}

pub async fn delete<C: Context>(ctx: &C, session: &Session) -> ServiceResult<()> {
    // TODO: if the user has multiple sessions,
    // make another session their primary session

    channels::leave_all(ctx, session.session_id).await?;
    spectators::leave(ctx, session, None).await?;
    spectators::close(ctx, session.session_id).await?;
    multiplayer::leave(ctx, session, None).await?;

    presences::delete(ctx, session.user_id).await?;
    sessions::delete(ctx, session.session_id, session.user_id, &session.username).await?;
    streams::clear_stream(ctx, StreamName::User(session.session_id)).await?;
    streams::leave_all(ctx, session.session_id).await?;

    // notify everyone
    let logout_notification = UserLogout::new(session.user_id as _);
    streams::broadcast_message(ctx, StreamName::Main, logout_notification, None, None).await?;
    Ok(())
}

pub async fn set_private_dms(
    ctx: &RequestContext,
    session: &Session,
    private_dms: bool,
) -> ServiceResult<()> {
    match sessions::set_private_dms(ctx, session.as_entity(), private_dms).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

/// Silences the given session for the given amount of seconds.
pub async fn silence<C: Context>(
    ctx: &C,
    session: &mut Session,
    silence_seconds: i64,
) -> ServiceResult<()> {
    session.silence_end = Some(chrono::Utc::now() + TimeDelta::seconds(silence_seconds));
    match sessions::update(ctx, session.as_entity()).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}
