use crate::common::context::Context;
use redis::AsyncCommands;
use std::ops::DerefMut;
use uuid::Uuid;

const SPECTATING_KEY: &'static str = "akatsuki:bancho:sessions:spectating";
pub async fn fetch_spectating<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> anyhow::Result<Option<Uuid>> {
    let mut redis = ctx.redis().await?;
    Ok(redis.hget(SPECTATING_KEY, session_id).await?)
}

pub async fn remove_spectating<C: Context>(ctx: &C, session_id: Uuid) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    Ok(redis.hdel(SPECTATING_KEY, session_id).await?)
}

fn make_key(host_session_id: Uuid) -> String {
    format!("akatsuki:bancho:spectator:{host_session_id}")
}

pub async fn add_member<C: Context>(
    ctx: &C,
    host_session_id: Uuid,
    session_id: Uuid,
    user_id: i64,
) -> anyhow::Result<usize> {
    let key = make_key(host_session_id);

    let mut redis = ctx.redis().await?;
    let member_count: [usize; 1] = redis::pipe()
        .hset(SPECTATING_KEY, session_id, host_session_id)
        .ignore()
        .sadd(&key, user_id)
        .ignore()
        .scard(key)
        .query_async(redis.deref_mut())
        .await?;
    Ok(member_count[0])
}

pub async fn remove_member<C: Context>(
    ctx: &C,
    host_session_id: Uuid,
    session_id: Uuid,
    user_id: i64,
) -> anyhow::Result<usize> {
    let key = make_key(host_session_id);

    let mut redis = ctx.redis().await?;
    let member_count: [usize; 1] = redis::pipe()
        .hdel(SPECTATING_KEY, session_id)
        .ignore()
        .srem(&key, user_id)
        .scard(key)
        .query_async(redis.deref_mut())
        .await?;
    Ok(member_count[0])
}

pub async fn fetch_all_members<C: Context>(
    ctx: &C,
    host_session_id: Uuid,
) -> anyhow::Result<Vec<i64>> {
    let mut redis = ctx.redis().await?;
    let key = make_key(host_session_id);
    Ok(redis.smembers(key).await?)
}

pub async fn remove_members<C: Context>(ctx: &C, host_session_id: Uuid) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let key = make_key(host_session_id);
    Ok(redis.del(key).await?)
}
