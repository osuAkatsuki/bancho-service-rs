use crate::common::context::Context;
use crate::entities::channels::ChannelName;
use crate::entities::streams::{MessageInfo, StreamMessage, StreamReadMessage, StreamReadReply};
use hashbrown::HashMap;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use redis::streams::{StreamRangeReply, StreamTrimOptions, StreamTrimmingMode};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Copy, Clone)]
pub enum StreamName<'a> {
    /// This stream is used to enqueue to a session
    /// Attached is the session_id
    User(Uuid),
    /// This stream is used to broadcast global events
    Main,
    /// This stream is used to broadcast multiplayer updates
    Lobby,
    /// This stream is used to broadcast to Akatsuki+ users
    Donator,
    /// This stream is used to broadcast to staff members
    Staff,
    /// This stream is used to broadcast to developers
    Dev,
    Channel(ChannelName<'a>),
    /// This stream is used to broadcast to spectators
    /// Attached is the session_id of the host
    Spectator(Uuid),
    /// This stream is used to broadcast to multiplayer matches
    Multiplayer(i64),
    /// This stream is used to broadcast to active multiplayer games
    Multiplaying(i64),
}

impl Display for StreamName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamName::User(session_id) => write!(f, "user:{}", session_id),
            StreamName::Main => write!(f, "main"),
            StreamName::Lobby => write!(f, "lobby"),
            StreamName::Donator => write!(f, "donator"),
            StreamName::Staff => write!(f, "staff"),
            StreamName::Dev => write!(f, "dev"),
            StreamName::Channel(channel_name) => write!(f, "channel:{channel_name}"),
            StreamName::Spectator(session_id) => write!(f, "spectator:{}", session_id),
            StreamName::Multiplayer(match_id) => write!(f, "multiplayer:{}", match_id),
            StreamName::Multiplaying(match_id) => write!(f, "multiplayer:{}:playing", match_id),
        }
    }
}

const BASE_KEY: &str = "akatsuki:bancho:streams";
const ALL_KEY: &str = const_str::concat!(BASE_KEY, ":*");
fn make_key(stream_name: StreamName) -> String {
    format!("{BASE_KEY}:{stream_name}")
}

fn make_offsets_key<T: Display>(session_id: T) -> String {
    format!("akatsuki:bancho:sessions:{session_id}:stream_offsets")
}

pub async fn fetch_all<C: Context>(ctx: &C) -> anyhow::Result<Vec<String>> {
    let mut redis = ctx.redis().await?;
    let mut iter: redis::AsyncIter<String> = redis.scan_match(ALL_KEY).await?;
    let mut keys = vec![];
    while let Some(stream_name) = iter.next_item().await {
        keys.push(stream_name);
    }
    Ok(keys)
}

pub async fn broadcast_data<C: Context + ?Sized>(
    ctx: &C,
    stream_name: StreamName<'_>,
    data: &[u8],
    info: MessageInfo,
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let key = make_key(stream_name);
    let message = StreamMessage::new(data, info);
    let _: () = redis.xadd(key, "*", &message.items()).await?;
    Ok(())
}

pub async fn read_pending_messages<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> anyhow::Result<Vec<StreamReadMessage>> {
    let mut redis = ctx.redis().await?;
    let mut offsets = get_offsets(&mut redis, session_id).await?;

    let streams: Vec<&String> = offsets.keys().collect();
    let ids: Vec<&String> = offsets.values().collect();
    let reply: Option<StreamReadReply> = redis.xread(&streams, &ids).await?;
    match reply {
        None => Ok(vec![]),
        Some(reply) => {
            let messages = reply
                .streams
                .into_iter()
                .flat_map(|stream| {
                    if let Some(last_id) = stream.messages.last() {
                        offsets.insert(stream.stream_name, last_id.message_id.clone());
                    } else {
                        offsets.remove(&stream.stream_name);
                    }

                    stream.messages
                })
                .collect::<Vec<_>>();

            // Update the users' streams offsets after reading messages from stream
            set_offsets(&mut redis, session_id, offsets).await?;
            Ok(messages)
        }
    }
}

pub async fn is_joined<C: Context>(
    ctx: &C,
    session_id: Uuid,
    stream_name: StreamName<'_>,
) -> anyhow::Result<bool> {
    let mut redis = ctx.redis().await?;
    let key = make_key(stream_name);
    let offsets_key = make_offsets_key(session_id);
    Ok(redis.hexists(offsets_key, key).await?)
}

pub async fn set_offset<C: Context>(
    ctx: &C,
    session_id: Uuid,
    stream_name: StreamName<'_>,
    id: String,
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let key = make_key(stream_name);
    let offsets_key = make_offsets_key(session_id);
    Ok(redis.hset(offsets_key, key, id).await?)
}

pub async fn remove_offset<C: Context>(
    ctx: &C,
    session_id: Uuid,
    stream_name: StreamName<'_>,
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let key = make_key(stream_name);
    let offsets_key = make_offsets_key(session_id);
    Ok(redis.hdel(offsets_key, key).await?)
}

pub async fn remove_offsets<C: Context>(ctx: &C, session_id: Uuid) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let offsets_key = make_offsets_key(session_id);
    Ok(redis.del(offsets_key).await?)
}

pub async fn clear_stream<C: Context>(ctx: &C, stream_name: StreamName<'_>) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let key = make_key(stream_name);
    Ok(redis.del(key).await?)
}

pub async fn get_latest_message_id<C: Context>(
    ctx: &C,
    stream_name: StreamName<'_>,
) -> anyhow::Result<String> {
    let mut redis = ctx.redis().await?;
    let key = make_key(stream_name);
    let message_ids: StreamRangeReply = redis.xrevrange_count(key, "+", "-", 1).await?;
    match message_ids.ids.is_empty() {
        true => Ok("0-0".to_string()),
        false => Ok(message_ids.ids[0].id.clone()),
    }
}

pub async fn trim_messages<C: Context>(ctx: &C, key: &str, min_id: &str) -> anyhow::Result<usize> {
    let mut redis = ctx.redis().await?;
    let removed_count = redis
        .xtrim_options(
            key,
            &StreamTrimOptions::minid(StreamTrimmingMode::Exact, min_id),
        )
        .await?;
    Ok(removed_count)
}

// utility
async fn get_offsets(
    redis: &mut MultiplexedConnection,
    session_id: Uuid,
) -> anyhow::Result<HashMap<String, String>> {
    let offsets_key = make_offsets_key(&session_id);
    Ok(redis.hgetall(offsets_key).await?)
}

async fn set_offsets(
    redis: &mut MultiplexedConnection,
    session_id: Uuid,
    offsets: HashMap<String, String>,
) -> anyhow::Result<()> {
    let offsets_key = make_offsets_key(&session_id);
    let items: Vec<(&str, &str)> = offsets
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    Ok(redis.hset_multiple(offsets_key, items.as_slice()).await?)
}
