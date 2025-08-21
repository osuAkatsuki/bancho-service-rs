use crate::common::context::Context;
use crate::entities::channels::{Channel, ChannelName};
use redis::AsyncCommands;
use std::ops::DerefMut;
use uuid::Uuid;

const TABLE_NAME: &str = "bancho_channels";
const READ_FIELDS: &str = "id, name, description, public_read, public_write, status";

pub async fn fetch_one<C: Context>(ctx: &C, channel_name: &str) -> sqlx::Result<Channel> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " WHERE name = ?"
    );
    sqlx::query_as(QUERY)
        .bind(channel_name)
        .fetch_one(ctx.db())
        .await
}

pub async fn fetch_all<C: Context>(ctx: &C) -> sqlx::Result<Vec<Channel>> {
    const QUERY: &str = const_str::concat!("SELECT ", READ_FIELDS, " FROM ", TABLE_NAME);
    sqlx::query_as(QUERY).fetch_all(ctx.db()).await
}

fn make_channel_members_key(channel_name: &ChannelName) -> String {
    format!("akatsuki:bancho:channels:{channel_name}:members")
}

fn make_session_channels_key(session_id: Uuid) -> String {
    format!("akatsuki:bancho:session:{session_id}:channels")
}

pub async fn fetch_session_channels<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> anyhow::Result<Vec<String>> {
    let mut redis = ctx.redis().await?;
    let session_channels_key = make_session_channels_key(session_id);
    Ok(redis.smembers(session_channels_key).await?)
}

pub async fn member_count<C: Context>(
    ctx: &C,
    channel_name: ChannelName<'_>,
) -> anyhow::Result<usize> {
    let mut redis = ctx.redis().await?;
    let members_key = make_channel_members_key(&channel_name);
    Ok(redis.scard(members_key).await?)
}

pub async fn join<C: Context>(
    ctx: &C,
    session_id: Uuid,
    channel_name: ChannelName<'_>,
) -> anyhow::Result<usize> {
    let mut redis = ctx.redis().await?;
    let session_channels_key = make_session_channels_key(session_id);
    let members_key = make_channel_members_key(&channel_name);
    let member_count: [usize; 1] = redis::pipe()
        .atomic()
        .sadd(session_channels_key, channel_name.to_string())
        .ignore()
        .sadd(&members_key, session_id.to_string())
        .ignore()
        .scard(members_key)
        .query_async(redis.deref_mut())
        .await?;
    Ok(member_count[0])
}

pub async fn leave<C: Context>(
    ctx: &C,
    session_id: Uuid,
    channel_name: ChannelName<'_>,
) -> anyhow::Result<usize> {
    let mut redis = ctx.redis().await?;
    let session_channels_key = make_session_channels_key(session_id);
    let members_key = make_channel_members_key(&channel_name);
    let member_count: [usize; 1] = redis::pipe()
        .atomic()
        .srem(session_channels_key, channel_name.to_string())
        .ignore()
        .srem(&members_key, session_id.to_string())
        .ignore()
        .scard(members_key)
        .query_async(redis.deref_mut())
        .await?;
    Ok(member_count[0])
}