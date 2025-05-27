use crate::adapters::ip_api;
use crate::api::RequestContext;
use crate::common::error::{AppError, ServiceResult, unexpected, unwrap_expect};
use crate::entities::sessions::CreateSessionArgs;
use crate::models::Gamemode;
use crate::models::bancho::LoginArgs;
use crate::models::presences::Presence;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::models::users::User;
use crate::repositories::streams::StreamName;
use crate::repositories::{sessions, users};
use crate::usecases::{channels, presences, stats, streams};
use bancho_protocol::structures::Country;
use tracing::error;
use uuid::Uuid;

pub async fn create(
    ctx: &RequestContext,
    args: LoginArgs,
) -> ServiceResult<(Session, User, Presence)> {
    let user = unwrap_expect! {
        users::fetch_one_by_username(ctx, &args.identifier).await,
        Err(sqlx::Error::RowNotFound) => return Err(AppError::SessionsInvalidCredentials)
    };

    if !bcrypt::verify(&args.secret, &user.password_md5)? {
        return Err(AppError::SessionsInvalidCredentials);
    }

    let user = User::try_from(user)?;
    if !user.privileges.contains(Privileges::CanLogin) {
        return Err(AppError::SessionsLoginForbidden);
    }

    // TODO: implement this
    // args.client_info

    let stats = stats::fetch_one(ctx, user.user_id, Gamemode::Standard).await?;
    let rank = stats::fetch_global_rank(ctx, user.user_id, Gamemode::Standard).await?;

    let ip_address = ctx.request_ip.ip_addr;
    let (connecting_country, latitude, longitude) = match ip_api::get_ip_info(ip_address).await {
        Ok(location) => {
            let country =
                Country::try_from_iso3166_2(&location.country_code).unwrap_or(user.country);
            (country, location.latitude, location.longitude)
        }
        Err(e) => {
            error!("Error getting location for session: {e:?}");
            (user.country, 0.0, 0.0)
        }
    };

    let user_id = user.user_id;
    let privileges = user.privileges.bits();
    let session = sessions::create(
        ctx,
        CreateSessionArgs {
            user_id,
            privileges,
            ip_address,
        },
    )
    .await?;
    // after creating the presence, the user becomes visible on the users panel
    let presence = presences::create_default(
        ctx,
        user.user_id,
        stats.ranked_score,
        stats.total_score,
        stats.avg_accuracy,
        stats.playcount,
        stats.pp,
        rank,
        connecting_country,
        latitude,
        longitude,
        args.client_info.utc_offset,
    )
    .await?;
    Ok((Session::from(session), user, presence))
}

pub async fn fetch_one(ctx: &RequestContext, session_id: Uuid) -> ServiceResult<Session> {
    match sessions::fetch_one(ctx, session_id).await {
        Ok(Some(session)) if session.is_expired() => {
            sessions::delete(ctx, session_id).await?;
            Err(AppError::SessionsNotFound)
        }
        Ok(Some(session)) => Ok(Session::from(session)),
        Ok(None) => try_migrate_session(ctx, session_id).await,
        Err(e) => unexpected(e),
    }
}

// TODO: remove this once all sessions have been migrated
pub async fn try_migrate_session(ctx: &RequestContext, session_id: Uuid) -> ServiceResult<Session> {
    match sessions::fetch_one_fallback(ctx, session_id).await {
        Ok(Some(session)) => {
            sessions::delete_fallback(ctx, session).await?;
            Err(AppError::SessionsNeedsMigration)
        }
        Ok(None) => Err(AppError::SessionsNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn extend(ctx: &RequestContext, session_id: Uuid) -> ServiceResult<Session> {
    let session = fetch_one(ctx, session_id).await?;
    match sessions::extend(ctx, session.into()).await {
        Ok(session) => Ok(Session::from(session)),
        Err(e) => unexpected(e),
    }
}

pub async fn delete(ctx: &RequestContext, session: &Session) -> ServiceResult<()> {
    channels::leave_all(ctx, session.session_id).await?;
    /*spectator::leave_current(ctx, session.session_id).await?;
    spectator::close(ctx, session.session_id).await?;
    multiplayer::leave_current(ctx, session.session_id).await?;*/

    presences::delete(ctx, session.user_id).await?;
    sessions::delete(ctx, session.session_id).await?;
    streams::clear_stream(ctx, StreamName::User(session.session_id)).await?;
    streams::leave_all(ctx, session.session_id).await?;
    Ok(())
}
