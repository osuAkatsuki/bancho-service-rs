use crate::common::chat;
use crate::common::chat::safe_username;
use crate::common::context::Context;
use crate::common::redis_json::Json;
use crate::entities::sessions::{CreateSessionArgs, FallbackSession, Session};
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
        primary: args.primary,
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

pub async fn extend<C: Context>(ctx: &C, mut session: Session) -> anyhow::Result<Session> {
    session.updated_at = chrono::Utc::now();
    update(ctx, session).await
}

pub async fn update<C: Context>(ctx: &C, session: Session) -> anyhow::Result<Session> {
    let mut redis = ctx.redis().await?;
    let _: () = redis
        .hset(SESSIONS_KEY, session.session_id, Json(&session))
        .await?;
    Ok(session)
}

pub async fn fetch_random_non_primary<C: Context>(
    ctx: &C,
    user_id: i64,
) -> anyhow::Result<Option<Session>> {
    let mut redis = ctx.redis().await?;
    let user_id_key = make_id_key(user_id);
    // fetching 2 random sessions guarantees one of them is not a primary session
    let session_ids: Vec<Uuid> = redis.srandmember_multiple(user_id_key, 2).await?;
    if session_ids.len() != 2 {
        return Ok(None);
    }
    let mut sessions = fetch_many(ctx, &session_ids).await?;
    Ok(sessions.find(|x| !x.primary))
}

pub async fn delete<C: Context>(
    ctx: &C,
    session_id: Uuid,
    user_id: i64,
    username: &str,
    new_primary_session: Option<Session>,
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
    if let Some(mut new_primary_session) = new_primary_session {
        new_primary_session.primary = true;
        pipe.hset(
            SESSIONS_KEY,
            new_primary_session.session_id,
            Json(new_primary_session),
        )
        .ignore();
    }
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

const FALLBACK_SESSIONS_KEY: &str = "bancho:tokens:json";

fn make_fallback_user_id_key(user_id: i64) -> String {
    format!("bancho:tokens:ids:{user_id}")
}

fn make_fallback_username_key(username: &str) -> String {
    let safe_username = chat::safe_username(username);
    format!("bancho:tokens:names:{safe_username}")
}

fn make_fallback_key(session_id: &str) -> String {
    format!("bancho:tokens:{session_id}")
}

fn make_fallback_channels_key(session_id: &str) -> String {
    format!("{}:channels", make_fallback_key(session_id))
}

fn make_fallback_spectators_key(session_id: &str) -> String {
    format!("{}:spectators", make_fallback_key(session_id))
}

fn make_fallback_streams_key(session_id: &str) -> String {
    format!("{}:streams", make_fallback_key(session_id))
}

fn make_fallback_stream_offsets_key(session_id: &str) -> String {
    format!("{}:stream_offsets", make_fallback_key(session_id))
}

fn make_fallback_message_history_key(session_id: &str) -> String {
    format!("{}:message_history", make_fallback_key(session_id))
}

fn make_fallback_sent_away_messages_key(session_id: &str) -> String {
    format!("{}:sent_away_messages", make_fallback_key(session_id))
}

fn make_fallback_processing_lock_key(session_id: &str) -> String {
    format!("{}:processing_lock", make_fallback_key(session_id))
}

fn make_fallback_user_stream_key(session_id: &str) -> String {
    format!("bancho:streams:tokens/{session_id}:messages")
}

fn make_fallback_user_stream_messages_key(session_id: &str) -> String {
    format!("bancho:streams:tokens/{session_id}:messages:messages")
}

pub async fn fetch_one_fallback<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> anyhow::Result<Option<FallbackSession>> {
    let mut redis = ctx.redis().await?;
    let session: Option<Json<FallbackSession>> = redis
        .hget(FALLBACK_SESSIONS_KEY, session_id.to_string())
        .await?;
    Ok(session.map(Json::into_inner))
}

pub async fn delete_fallback<C: Context>(ctx: &C, session: FallbackSession) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let id_key = make_fallback_user_id_key(session.user_id);
    let name_key = make_fallback_username_key(&session.username);
    let channels_key = make_fallback_channels_key(&session.token_id);
    let spectators_key = make_fallback_spectators_key(&session.token_id);
    let streams_key = make_fallback_streams_key(&session.token_id);
    let stream_offsets_key = make_fallback_stream_offsets_key(&session.token_id);
    let user_stream_key = make_fallback_user_stream_key(&session.token_id);
    let user_stream_messages_key = make_fallback_user_stream_messages_key(&session.token_id);
    let msg_history_key = make_fallback_message_history_key(&session.token_id);
    let afk_msgs_key = make_fallback_sent_away_messages_key(&session.token_id);
    let processing_lock_key = make_fallback_processing_lock_key(&session.token_id);
    redis::pipe()
        .atomic()
        .hdel(FALLBACK_SESSIONS_KEY, session.token_id)
        .ignore()
        .del(id_key)
        .ignore()
        .del(name_key)
        .ignore()
        .del(channels_key)
        .ignore()
        .del(spectators_key)
        .ignore()
        .del(streams_key)
        .ignore()
        .del(stream_offsets_key)
        .ignore()
        .del(user_stream_key)
        .ignore()
        .del(user_stream_messages_key)
        .ignore()
        .del(msg_history_key)
        .ignore()
        .del(afk_msgs_key)
        .ignore()
        .del(processing_lock_key)
        .ignore()
        .exec_async(redis.deref_mut())
        .await?;
    Ok(())
}
