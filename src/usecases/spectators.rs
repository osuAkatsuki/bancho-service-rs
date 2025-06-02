use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::entities::channels::ChannelName;
use crate::models::sessions::Session;
use crate::repositories::spectators;
use crate::repositories::streams::StreamName;
use crate::usecases::{channels, sessions, streams};
use bancho_protocol::messages::server::{
    ChannelKick, FellowSpectatorJoined, FellowSpectatorLeft, SpectatorJoined, SpectatorLeft,
};
use uuid::Uuid;

pub async fn fetch_spectating<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> ServiceResult<Option<Uuid>> {
    match spectators::fetch_spectating(ctx, session_id).await {
        Ok(spectating) => Ok(spectating),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_all_members<C: Context>(
    ctx: &C,
    host_session_id: Uuid,
) -> ServiceResult<Vec<i64>> {
    match spectators::fetch_all_members(ctx, host_session_id).await {
        Ok(members) => Ok(members),
        Err(e) => unexpected(e),
    }
}

pub async fn join<C: Context>(ctx: &C, session: &Session, host_id: i64) -> ServiceResult<Vec<i64>> {
    if let Some(host_session_id) = spectators::fetch_spectating(ctx, session.session_id).await? {
        leave(ctx, session, Some(host_session_id)).await?;
    }

    let host_session = sessions::fetch_one_by_user_id(ctx, host_id).await?;
    if !session.is_publicly_visible() {
        return Err(AppError::InteractionBlocked);
    }

    let member_count = spectators::add_member(
        ctx,
        host_session.session_id,
        session.session_id,
        session.user_id,
    )
    .await?;
    if member_count == 0 {
        tracing::error!("Unexpected Spectators Member Count of 0");
    }

    let stream_name = StreamName::Spectator(host_session.session_id);
    let channel_name = ChannelName::Spectator(host_session.session_id);
    streams::join(ctx, session.session_id, stream_name).await?;
    channels::join(ctx, session, channel_name).await?;

    // Notify the host and other spectators about allat
    let host_stream_name = StreamName::User(host_session.session_id);
    let host_notification = SpectatorJoined {
        user_id: session.user_id as _,
    };
    streams::broadcast_message(ctx, host_stream_name, host_notification, None, None).await?;

    if member_count <= 1 {
        // we are the first spectator
        // Join the host to their spectator updates stream
        streams::join(ctx, host_session.session_id, stream_name).await?;
        channels::join(ctx, &host_session, channel_name).await?;
        Ok(vec![session.user_id])
    } else {
        let spectator_notification = FellowSpectatorJoined {
            user_id: session.user_id as _,
        };
        streams::broadcast_message(
            ctx,
            stream_name,
            spectator_notification,
            Some(vec![session.session_id, host_session.session_id]),
            None,
        )
        .await?;
        let members = spectators::fetch_all_members(ctx, host_session.session_id).await?;
        Ok(members)
    }
}

pub async fn leave<C: Context>(
    ctx: &C,
    session: &Session,
    host_session_id: Option<Uuid>,
) -> ServiceResult<usize> {
    let host_session_id = match host_session_id {
        Some(host_session_id) => host_session_id,
        None => {
            let spectating = spectators::fetch_spectating(ctx, session.session_id).await?;
            match spectating {
                Some(host_session_id) => host_session_id,
                None => return Ok(0),
            }
        }
    };
    let member_count =
        spectators::remove_member(ctx, host_session_id, session.session_id, session.user_id)
            .await?;

    let channel_name = ChannelName::Spectator(host_session_id);
    channels::leave(ctx, session.session_id, channel_name).await?;

    let stream_name = StreamName::Spectator(host_session_id);
    streams::leave(ctx, session.session_id, stream_name).await?;

    // Notify the host and other spectators about allat
    let host_stream_name = StreamName::User(host_session_id);
    let host_notification = SpectatorLeft {
        user_id: session.user_id as _,
    };
    streams::broadcast_message(ctx, host_stream_name, host_notification, None, None).await?;

    if member_count == 0 {
        // we were the last spectating user
        // Remove the host from the chat channel and also from the spectator stream
        streams::leave(ctx, host_session_id, stream_name).await?;
        channels::leave(ctx, host_session_id, channel_name).await?;
        streams::clear_stream(ctx, stream_name).await?;
        streams::broadcast_message(
            ctx,
            host_stream_name,
            ChannelKick { name: "#spectator" },
            None,
            None,
        )
        .await?;
    } else {
        // Notify other spectators that we have left
        let spectator_notification = FellowSpectatorLeft {
            user_id: session.user_id as _,
        };
        streams::broadcast_message(
            ctx,
            stream_name,
            spectator_notification,
            Some(vec![host_session_id]),
            None,
        )
        .await?;
    }
    Ok(member_count)
}

pub async fn close<C: Context>(ctx: &C, session_id: Uuid) -> ServiceResult<()> {
    let members = fetch_all_members(ctx, session_id).await?;
    if members.is_empty() {
        return Ok(());
    }

    let sessions = sessions::fetch_many_by_user_id(ctx, &members).await?;
    let channel_name = ChannelName::Spectator(session_id);
    let stream_name = StreamName::Spectator(session_id);
    for session_id in sessions {
        spectators::remove_spectating(ctx, session_id).await?;
        channels::leave(ctx, session_id, channel_name).await?;
        streams::leave(ctx, session_id, stream_name).await?;
    }

    spectators::remove_members(ctx, session_id).await?;
    streams::clear_stream(ctx, stream_name).await?;

    Ok(())
}
