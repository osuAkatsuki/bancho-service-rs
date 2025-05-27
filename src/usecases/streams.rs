use crate::api::RequestContext;
use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::entities::streams::MessageInfo;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::streams;
use crate::repositories::streams::StreamName;
use bancho_protocol::messages::MessageArgs;
use bancho_protocol::serde::BinarySerialize;
use chrono::TimeDelta;
use tracing::error;
use uuid::Uuid;

pub async fn broadcast_message<M: MessageArgs>(
    ctx: &RequestContext,
    stream_name: StreamName<'_>,
    args: M,
    excluded_session_ids: Option<Vec<Uuid>>,
    read_privileges: Option<Privileges>,
) -> ServiceResult<()> {
    let msg = args.as_message().serialize();
    broadcast_data(
        ctx,
        stream_name,
        &msg,
        excluded_session_ids,
        read_privileges,
    )
    .await
}

pub async fn broadcast_data(
    ctx: &RequestContext,
    stream_name: StreamName<'_>,
    data: &[u8],
    excluded_session_ids: Option<Vec<Uuid>>,
    read_privileges: Option<Privileges>,
) -> ServiceResult<()> {
    match streams::broadcast_data(
        ctx,
        stream_name,
        data,
        MessageInfo {
            excluded_session_ids,
            read_privileges: read_privileges.map(|privs| privs.bits()),
        },
    )
    .await
    {
        Ok(()) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn read_pending_data(ctx: &RequestContext, session: &Session) -> ServiceResult<Vec<u8>> {
    let messages = streams::read_pending_messages(ctx, session.session_id).await?;
    let mut pending_data = vec![];
    for msg in messages {
        let is_excluded = msg
            .info
            .excluded_session_ids
            .is_some_and(|excluded_session_ids| excluded_session_ids.contains(&session.session_id));
        let can_read = msg
            .info
            .read_privileges
            .is_none_or(|privs| session.has_all_privileges(Privileges::from_bits_retain(privs)));
        if can_read && !is_excluded {
            pending_data.extend(&msg.data);
        }
    }
    Ok(pending_data)
}

pub async fn join(
    ctx: &RequestContext,
    session_id: Uuid,
    stream_name: StreamName<'_>,
) -> ServiceResult<()> {
    let latest_message_id = streams::get_latest_message_id(ctx, stream_name).await?;
    streams::set_offset(ctx, session_id, stream_name, latest_message_id).await?;
    Ok(())
}

pub async fn leave(
    ctx: &RequestContext,
    session_id: Uuid,
    stream_name: StreamName<'_>,
) -> ServiceResult<()> {
    streams::remove_offset(ctx, session_id, stream_name).await?;
    Ok(())
}

pub async fn leave_all(ctx: &RequestContext, session_id: Uuid) -> ServiceResult<()> {
    match streams::remove_offsets(ctx, session_id).await {
        Ok(()) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn is_joined(
    ctx: &RequestContext,
    session_id: Uuid,
    stream_name: StreamName<'_>,
) -> ServiceResult<bool> {
    match streams::is_joined(ctx, session_id, stream_name).await {
        Ok(is_joined) => Ok(is_joined),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_all<C: Context>(ctx: &C) -> ServiceResult<Vec<String>> {
    match streams::fetch_all(ctx).await {
        Ok(streams) => Ok(streams),
        Err(e) => unexpected(e),
    }
}

/// Trims stream messages to the given ttl
pub async fn trim_stream<C: Context>(
    ctx: &C,
    stream_key: &str,
    ttl_seconds: usize,
) -> ServiceResult<usize> {
    let now = chrono::Utc::now();
    let min_id = now - TimeDelta::seconds(ttl_seconds as _);
    let min_id = format!("{}-0", min_id.timestamp_millis());
    match streams::trim_messages(ctx, stream_key, &min_id).await {
        Ok(count) => Ok(count),
        Err(e) => unexpected(e),
    }
}

pub async fn trim_all_streams<C: Context>(
    ctx: &C,
    ttl_seconds: usize,
) -> ServiceResult<Vec<(String, usize)>> {
    let streams = fetch_all(ctx).await?;
    let mut results = Vec::with_capacity(streams.len());
    for stream in streams {
        match trim_stream(ctx, &stream, ttl_seconds).await {
            Ok(count) => results.push((stream, count)),
            Err(e) => error!(stream_name = stream, "Error trimming stream: {e:?}"),
        }
    }
    Ok(results)
}

pub async fn clear_stream(ctx: &RequestContext, stream_name: StreamName<'_>) -> ServiceResult<()> {
    match streams::clear_stream(ctx, stream_name).await {
        Ok(()) => Ok(()),
        Err(e) => unexpected(e),
    }
}
