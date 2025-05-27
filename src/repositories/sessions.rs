use crate::api::RequestContext;
use crate::common::chat;
use crate::common::redis_json::Json;
use crate::entities::sessions::{CreateSessionArgs, FallbackSession, Session};
use redis::AsyncCommands;
use std::ops::DerefMut;
use uuid::Uuid;

const SESSIONS_KEY: &str = "akatsuki:bancho:sessions";
pub async fn create(ctx: &RequestContext, args: CreateSessionArgs) -> anyhow::Result<Session> {
    let session = Session {
        session_id: Uuid::new_v4(),
        user_id: args.user_id,
        privileges: args.privileges,
        create_ip_address: args.ip_address,
        updated_at: chrono::Utc::now(),
    };
    let session_id = session.session_id.to_string();

    let mut redis = ctx.redis.get().await?;
    let _: () = redis.hset(SESSIONS_KEY, session_id, Json(&session)).await?;
    Ok(session)
}

pub async fn fetch_one(ctx: &RequestContext, session_id: Uuid) -> anyhow::Result<Option<Session>> {
    let mut redis = ctx.redis.get().await?;
    let session: Option<Json<Session>> = redis.hget(SESSIONS_KEY, session_id.to_string()).await?;
    Ok(session.map(Json::into_inner))
}

pub async fn extend(ctx: &RequestContext, mut session: Session) -> anyhow::Result<Session> {
    session.updated_at = chrono::Utc::now();
    update(ctx, session).await
}

pub async fn update(ctx: &RequestContext, session: Session) -> anyhow::Result<Session> {
    let session_id = session.session_id.to_string();
    let mut redis = ctx.redis.get().await?;
    let _: () = redis.hset(SESSIONS_KEY, session_id, Json(&session)).await?;
    Ok(session)
}

pub async fn delete(ctx: &RequestContext, session_id: Uuid) -> anyhow::Result<()> {
    let mut redis = ctx.redis.get().await?;
    let _: () = redis.hdel(SESSIONS_KEY, session_id.to_string()).await?;
    Ok(())
}

pub async fn count(ctx: &RequestContext) -> anyhow::Result<usize> {
    let mut redis = ctx.redis.get().await?;
    let count: usize = redis.hlen(SESSIONS_KEY).await?;
    Ok(count)
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

pub async fn fetch_one_fallback(
    ctx: &RequestContext,
    session_id: Uuid,
) -> anyhow::Result<Option<FallbackSession>> {
    let mut redis = ctx.redis.get().await?;
    let session: Option<Json<FallbackSession>> = redis
        .hget(FALLBACK_SESSIONS_KEY, session_id.to_string())
        .await?;
    Ok(session.map(Json::into_inner))
}

pub async fn delete_fallback(ctx: &RequestContext, session: FallbackSession) -> anyhow::Result<()> {
    let mut redis = ctx.redis.get().await?;
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
