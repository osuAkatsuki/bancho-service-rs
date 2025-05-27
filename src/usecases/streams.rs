use crate::api::RequestContext;
use crate::common::error::{ServiceResult, unexpected};
use crate::entities::streams::MessageInfo;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::streams;
use crate::repositories::streams::StreamName;
use bancho_protocol::messages::MessageArgs;
use bancho_protocol::serde::BinarySerialize;
use uuid::Uuid;

pub async fn fetch_all(ctx: &RequestContext) -> ServiceResult<Vec<String>> {
    match streams::fetch_all(ctx).await {
        Ok(streams) => Ok(streams),
        Err(e) => unexpected(e),
    }
}

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
    let latest_message_id = streams::get_latest_message_id(ctx, stream_name.clone()).await?;
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

pub async fn clear_stream(ctx: &RequestContext, stream_name: StreamName<'_>) -> ServiceResult<()> {
    match streams::clear_stream(ctx, stream_name).await {
        Ok(()) => Ok(()),
        Err(e) => unexpected(e),
    }
}
