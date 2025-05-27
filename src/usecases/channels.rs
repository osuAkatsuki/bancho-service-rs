use crate::api::RequestContext;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::models::channels::Channel;
use crate::models::sessions::Session;
use crate::repositories::channels;
use crate::repositories::streams::StreamName;
use crate::usecases::streams;
use bancho_protocol::messages::server::ChannelInfo;
use uuid::Uuid;

pub async fn fetch_one(ctx: &RequestContext, channel_name: &str) -> ServiceResult<Channel> {
    match channels::fetch_one(ctx, channel_name).await {
        Ok(channel) => Ok(Channel::from(channel)),
        Err(sqlx::Error::RowNotFound) => Err(AppError::ChannelsNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_all(ctx: &RequestContext) -> ServiceResult<Vec<Channel>> {
    match channels::fetch_all(ctx).await {
        Ok(channels) => Ok(channels
            .into_iter()
            .map(|channel| Channel::from(channel))
            .collect()),
        Err(e) => unexpected(e),
    }
}

pub async fn join(
    ctx: &RequestContext,
    session: &Session,
    channel_name: &str,
) -> ServiceResult<(Channel, usize)> {
    let channel = fetch_one(ctx, channel_name).await?;
    if !channel.can_read(session.privileges) {
        return Err(AppError::ChannelsUnauthorized);
    }

    let stream_name = StreamName::Channel(channel_name);
    streams::join(ctx, session.session_id, stream_name).await?;
    let member_count = channels::join(ctx, session.session_id, channel_name).await?;

    broadcast_channel_info_update(ctx, &channel, member_count).await?;

    Ok((channel, member_count))
}

pub async fn leave(
    ctx: &RequestContext,
    session_id: Uuid,
    channel_name: &str,
) -> ServiceResult<(Channel, usize)> {
    let channel = fetch_one(ctx, channel_name).await?;
    let stream_name = StreamName::Channel(channel_name);
    streams::leave(ctx, session_id, stream_name).await?;
    let member_count = channels::leave(ctx, session_id, channel_name).await?;

    broadcast_channel_info_update(ctx, &channel, member_count).await?;

    Ok((channel, member_count))
}

pub async fn leave_all(ctx: &RequestContext, session_id: Uuid) -> ServiceResult<()> {
    let channels = channels::fetch_session_channels(ctx, session_id).await?;
    for channel in channels {
        leave(ctx, session_id, &channel).await?;
    }
    Ok(())
}

pub async fn member_count(ctx: &RequestContext, channel_name: &str) -> ServiceResult<usize> {
    match channels::member_count(ctx, channel_name).await {
        Ok(member_count) => Ok(member_count),
        Err(e) => unexpected(e),
    }
}

// utility

async fn broadcast_channel_info_update(
    ctx: &RequestContext,
    channel: &Channel,
    member_count: usize,
) -> ServiceResult<()> {
    let update_stream = channel.get_update_stream_name();
    let priv_rule = match update_stream {
        StreamName::Donator => None,
        StreamName::Staff => None,
        StreamName::Dev => None,
        _ => Some(channel.read_privileges),
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
