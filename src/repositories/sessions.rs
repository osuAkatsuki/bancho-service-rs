use crate::common::chat;
use crate::common::chat::safe_username;
use crate::common::context::Context;
use crate::common::redis_json::Json;
use crate::entities::sessions::{CreateSessionArgs, FallbackSession, Session};
use chrono::TimeDelta;
use redis::AsyncCommands;
use std::ops::DerefMut;
use uuid::Uuid;

const SESSIONS_KEY: &str = "akatsuki:bancho:sessions";
const SESSIONS_USER_IDS_KEY: &str = "akatsuki:bancho:sessions:user_ids";
const SESSIONS_USERNAMES_KEY: &str = "akatsuki:bancho:sessions:usernames";
pub async fn create<C: Context>(ctx: &C, args: CreateSessionArgs) -> anyhow::Result<Session> {
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
    let safe_username = safe_username(&session.username);
    let mut redis = ctx.redis().await?;
    redis::pipe()
        .atomic()
        .hset(SESSIONS_KEY, session.session_id, Json(&session))
        .ignore()
        .hset(SESSIONS_USER_IDS_KEY, session.user_id, session.session_id)
        .ignore()
        .hset(SESSIONS_USERNAMES_KEY, safe_username, session.session_id)
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

pub async fn fetch_one_by_user_id<C: Context>(
    ctx: &C,
    user_id: i64,
) -> anyhow::Result<Option<Session>> {
    let mut redis = ctx.redis().await?;
    let session_id: Option<Uuid> = redis.hget(SESSIONS_USER_IDS_KEY, user_id).await?;
    if let Some(session_id) = session_id {
        let session: Option<Json<Session>> = redis.hget(SESSIONS_KEY, session_id).await?;
        Ok(session.map(Json::into_inner))
    } else {
        Ok(None)
    }
}

pub async fn fetch_one_by_username<C: Context + ?Sized>(
    ctx: &C,
    username: &str,
) -> anyhow::Result<Option<Session>> {
    let mut redis = ctx.redis().await?;
    let safe_username = safe_username(username);
    let session_id: Option<Uuid> = redis.hget(SESSIONS_USERNAMES_KEY, safe_username).await?;
    if let Some(session_id) = session_id {
        let session: Option<Json<Session>> = redis.hget(SESSIONS_KEY, session_id).await?;
        Ok(session.map(Json::into_inner))
    } else {
        Ok(None)
    }
}

pub async fn fetch_many_by_user_id<C: Context>(
    ctx: &C,
    user_ids: &[i64],
) -> anyhow::Result<impl Iterator<Item = Uuid>> {
    let mut redis = ctx.redis().await?;
    let session_ids: Vec<Option<Uuid>> = redis::cmd("HMGET")
        .arg(SESSIONS_USER_IDS_KEY)
        .arg(user_ids)
        .query_async(redis.deref_mut())
        .await?;
    Ok(session_ids.into_iter().filter_map(|x| x))
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

pub async fn delete<C: Context + ?Sized>(
    ctx: &C,
    session_id: Uuid,
    user_id: i64,
    username: &str,
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let safe_username = safe_username(username);
    redis::pipe()
        .atomic()
        .hdel(SESSIONS_KEY, session_id)
        .ignore()
        .hdel(SESSIONS_USER_IDS_KEY, user_id)
        .ignore()
        .hdel(SESSIONS_USERNAMES_KEY, safe_username)
        .ignore()
        .exec_async(redis.deref_mut())
        .await?;
    Ok(())
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

pub async fn silence<C: Context>(
    ctx: &C,
    mut session: Session,
    silence_seconds: i64,
) -> anyhow::Result<Session> {
    session.silence_end = Some(chrono::Utc::now() + TimeDelta::seconds(silence_seconds));
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
