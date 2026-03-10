use crate::common::chat::safe_username;
use crate::common::context::{Context, PoolContext};
use crate::common::redis_json::Json;
use crate::entities::sessions::{CreateSessionArgs, Session};
use redis::AsyncCommands;
use std::ops::DerefMut;
use uuid::Uuid;

const SESSIONS_KEY: &str = "akatsuki:bancho:sessions";

fn make_id_key(user_id: i64) -> String {
    format!("akatsuki:bancho:sessions:user_ids:{user_id}")
}
fn make_username_key(username: &str) -> String {
    let safe_username = safe_username(username);
    format!("akatsuki:bancho:sessions:usernames:{safe_username}")
}

pub async fn create<C: Context>(ctx: &C, args: CreateSessionArgs) -> anyhow::Result<Session> {
    let mut redis = ctx.redis().await?;
    let session = Session {
        session_id: Uuid::new_v4(),
        user_id: args.user_id,
        username: args.username,
        privileges: args.privileges,
        create_ip_address: args.ip_address,
        private_dms: args.private_dms,
        silence_end: args.silence_end,
        updated_at: chrono::Utc::now(),
    };
    let user_id_key = make_id_key(args.user_id);
    let username_key = make_username_key(&session.username);
    redis::pipe()
        .atomic()
        .hset(SESSIONS_KEY, session.session_id, Json(&session))
        .ignore()
        .sadd(user_id_key, session.session_id)
        .ignore()
        .sadd(username_key, session.session_id)
        .ignore()
        .exec_async(redis.deref_mut())
        .await?;
    Ok(session)
}

pub async fn fetch_one<C: Context>(ctx: &C, session_id: Uuid) -> anyhow::Result<Option<Session>> {
    let mut redis = ctx.redis().await?;
    let session: Option<Json<Session>> = redis.hget(SESSIONS_KEY, session_id).await?;
    Ok(session.map(Json::into_inner))
}

pub async fn fetch_all<C: Context>(ctx: &C) -> anyhow::Result<impl Iterator<Item = Session>> {
    let mut redis = ctx.redis().await?;
    let sessions: Vec<Json<Session>> = redis.hvals(SESSIONS_KEY).await?;
    Ok(sessions.into_iter().map(Json::into_inner))
}

pub async fn fetch_many<C: Context>(
    ctx: &C,
    session_ids: &[Uuid],
) -> anyhow::Result<impl Iterator<Item = Session> + use<C>> {
    let sessions: Vec<Option<Json<Session>>> = match session_ids.is_empty() {
        true => vec![],
        false => {
            let mut redis = ctx.redis().await?;
            redis::cmd("HMGET")
                .arg(SESSIONS_KEY)
                .arg(&session_ids)
                .query_async(redis.deref_mut())
                .await?
        }
    };
    Ok(sessions.into_iter().filter_map(|x| x.map(Json::into_inner)))
}

pub async fn fetch_user_session_count<C: Context>(ctx: &C, user_id: i64) -> anyhow::Result<u64> {
    let mut redis = ctx.redis().await?;
    let user_id_key = make_id_key(user_id);
    Ok(redis.scard(user_id_key).await?)
}

pub async fn fetch_by_user_id<C: Context>(
    ctx: &C,
    user_id: i64,
) -> anyhow::Result<impl Iterator<Item = Session>> {
    let mut redis = ctx.redis().await?;
    let user_id_key = make_id_key(user_id);
    let session_ids: Vec<Uuid> = redis.smembers(user_id_key).await?;
    fetch_many(ctx, &session_ids).await
}

pub async fn fetch_by_username<C: Context>(
    ctx: &C,
    username: &str,
) -> anyhow::Result<impl Iterator<Item = Session>> {
    let mut redis = ctx.redis().await?;
    let username_key = make_username_key(username);
    let session_ids: Vec<Uuid> = redis.smembers(username_key).await?;
    fetch_many(ctx, &session_ids).await
}

pub async fn is_online<C: Context>(ctx: &C, user_id: i64) -> anyhow::Result<bool> {
    let mut redis = ctx.redis().await?;
    let user_id_key = make_id_key(user_id);
    Ok(redis.exists(user_id_key).await?)
}

pub async fn fetch_count<C: Context>(ctx: &C) -> anyhow::Result<u64> {
    let mut redis = ctx.redis().await?;
    Ok(redis.hlen(SESSIONS_KEY).await?)
}

pub async fn extend<C: Context>(ctx: &C, session: Session) -> anyhow::Result<Session> {
    update(ctx, session).await
}

pub async fn update<C: Context>(ctx: &C, mut session: Session) -> anyhow::Result<Session> {
    session.updated_at = chrono::Utc::now();
    let mut redis = ctx.redis().await?;
    let _: () = redis
        .hset(SESSIONS_KEY, session.session_id, Json(&session))
        .await?;
    Ok(session)
}

pub async fn delete<C: Context>(
    ctx: &C,
    session_id: Uuid,
    user_id: i64,
    username: &str,
) -> anyhow::Result<u64> {
    let mut redis = ctx.redis().await?;
    let user_id_key = make_id_key(user_id);
    let username_key = make_username_key(username);

    let mut pipe = redis::pipe();
    pipe.atomic()
        .hdel(SESSIONS_KEY, session_id)
        .ignore()
        .srem(&user_id_key, session_id)
        .ignore()
        .srem(username_key, session_id)
        .ignore()
        .scard(user_id_key);
    let size: [u64; 1] = pipe.query_async(redis.deref_mut()).await?;
    Ok(size[0])
}

pub async fn count<C: Context>(ctx: &C) -> anyhow::Result<usize> {
    let mut redis = ctx.redis().await?;
    let count: usize = redis.hlen(SESSIONS_KEY).await?;
    Ok(count)
}

pub async fn set_private_dms<C: Context>(
    ctx: &C,
    mut session: Session,
    private_dms: bool,
) -> anyhow::Result<Session> {
    session.private_dms = private_dms;
    update(ctx, session).await
}
