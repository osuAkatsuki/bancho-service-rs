use crate::common::context::Context;
use redis::AsyncCommands;
use std::ops::DerefMut;
use uuid::Uuid;
use crate::common::redis_json::Json;
use crate::entities::sessions::SessionIdentity;

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
    member_identity: SessionIdentity,
) -> anyhow::Result<usize> {
    let key = make_key(host_session_id);

    let mut redis = ctx.redis().await?;
    let member_count: [usize; 1] = redis::pipe()
        .atomic()
        .hset(SPECTATING_KEY, member_identity.session_id, host_session_id)
        .ignore()
        .sadd(&key, Json(member_identity))
        .ignore()
        .scard(key)
        .query_async(redis.deref_mut())
        .await?;
    Ok(member_count[0])
}

pub async fn remove_member<C: Context>(
    ctx: &C,
    host_session_id: Uuid,
    member_identity: SessionIdentity,
) -> anyhow::Result<usize> {
    let key = make_key(host_session_id);

    let mut redis = ctx.redis().await?;
    let member_count: [usize; 1] = redis::pipe()
        .atomic()
        .hdel(SPECTATING_KEY, member_identity.session_id)
        .ignore()
        .srem(&key, Json(member_identity))
        .ignore()
        .scard(key)
        .query_async(redis.deref_mut())
        .await?;
    Ok(member_count[0])
}

pub async fn fetch_all_members<C: Context>(
    ctx: &C,
    host_session_id: Uuid,
) -> anyhow::Result<impl Iterator<Item = SessionIdentity>> {
    let mut redis = ctx.redis().await?;
    let key = make_key(host_session_id);
    let identities: Vec<Json<SessionIdentity>> = redis.smembers(key).await?;
    Ok(identities.into_iter().map(Json::into_inner))
}

pub async fn remove_members<C: Context>(ctx: &C, host_session_id: Uuid) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let key = make_key(host_session_id);
    Ok(redis.del(key).await?)
}
