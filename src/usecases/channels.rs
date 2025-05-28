use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::entities::channels::ChannelName;
use crate::models::channels::Channel;
use crate::models::sessions::Session;
use crate::repositories::channels;
use crate::repositories::streams::StreamName;
use crate::usecases::{spectators, streams};
use bancho_protocol::messages::server::ChannelInfo;
use tracing::info;
use uuid::Uuid;

pub async fn get_channel_name<'a, C: Context>(
    ctx: &C,
    session: &Session,
    channel_name: &'a str,
) -> ServiceResult<ChannelName<'a>> {
    match channel_name {
        "#spectator" => {
            let host_session_id = spectators::fetch_spectating(ctx, session.session_id)
                .await?
                .ok_or(AppError::ChannelsUnauthorized)?;
            Ok(ChannelName::Spectator(host_session_id))
        }
        "#multiplayer" => {
            todo!()
        }
        channel_name => Ok(ChannelName::Chat(channel_name)),
    }
}

pub async fn fetch_one<C: Context>(
    ctx: &C,
    channel_name: ChannelName<'_>,
) -> ServiceResult<Channel> {
    match channel_name {
        ChannelName::Spectator(_) => Ok(Channel::spectator()),
        ChannelName::Multiplayer(_) => Ok(Channel::multiplayer()),
        ChannelName::Chat(channel_name) => match channels::fetch_one(ctx, channel_name).await {
            Ok(channel) => Ok(Channel::from(channel)),
            Err(sqlx::Error::RowNotFound) => Err(AppError::ChannelsNotFound),
            Err(e) => unexpected(e),
        },
    }
}

pub async fn fetch_all<C: Context>(ctx: &C) -> ServiceResult<Vec<Channel>> {
    match channels::fetch_all(ctx).await {
        Ok(channels) => Ok(channels
            .into_iter()
            .map(|channel| Channel::from(channel))
            .collect()),
        Err(e) => unexpected(e),
    }
}

pub async fn join<C: Context>(
    ctx: &C,
    session: &Session,
    channel_name: ChannelName<'_>,
) -> ServiceResult<(Channel, usize)> {
    let channel = fetch_one(ctx, channel_name).await?;
    if !channel.can_read(session.privileges) {
        return Err(AppError::ChannelsUnauthorized);
    }

    let stream_name = channel_name.get_message_stream();
    streams::join(ctx, session.session_id, stream_name).await?;
    let member_count = channels::join(ctx, session.session_id, channel_name).await?;
    info!(
        channel_name = channel_name.to_string(),
        member_count, "User joined channel."
    );

    broadcast_channel_info_update(ctx, channel_name, &channel, member_count).await?;

    Ok((channel, member_count))
}

pub async fn leave<C: Context>(
    ctx: &C,
    session_id: Uuid,
    channel_name: ChannelName<'_>,
) -> ServiceResult<(Channel, usize)> {
    let channel = fetch_one(ctx, channel_name).await?;
    let stream_name = channel_name.get_message_stream();
    streams::leave(ctx, session_id, stream_name).await?;
    let member_count = channels::leave(ctx, session_id, channel_name).await?;
    info!(
        channel_name = channel_name.to_string(),
        member_count, "User left channel."
    );

    broadcast_channel_info_update(ctx, channel_name, &channel, member_count).await?;

    Ok((channel, member_count))
}

pub async fn leave_all<C: Context>(ctx: &C, session_id: Uuid) -> ServiceResult<()> {
    let channels = channels::fetch_session_channels(ctx, session_id).await?;
    for channel in channels {
        leave(ctx, session_id, ChannelName::Chat(&channel)).await?;
    }
    Ok(())
}

pub async fn member_count<C: Context>(
    ctx: &C,
    channel_name: ChannelName<'_>,
) -> ServiceResult<usize> {
    match channels::member_count(ctx, channel_name).await {
        Ok(member_count) => Ok(member_count),
        Err(e) => unexpected(e),
    }
}

// utility

async fn broadcast_channel_info_update<C: Context>(
    ctx: &C,
    channel_name: ChannelName<'_>,
    channel: &Channel,
    member_count: usize,
) -> ServiceResult<()> {
    let update_stream = channel_name.get_update_stream();
    let priv_rule = match update_stream {
        StreamName::Main => Some(channel.read_privileges),
        _ => None,
    };

    streams::broadcast_message(
        ctx,
        update_stream,
        ChannelInfo {
            name: &channel.name,
            topic: &channel.description,
            user_count: member_count as _,
        },
        None,
        priv_rule,
    )
    .await?;

    Ok(())
}
