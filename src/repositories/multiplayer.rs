use crate::common::context::Context;
use crate::common::redis_json::Json;
use crate::entities::multiplayer::{MultiplayerMatch, MultiplayerMatchSlot};
use bancho_protocol::structures::SlotStatus;
use redis::AsyncCommands;
use std::ops::DerefMut;
use uuid::Uuid;

const KEY: &str = "akatsuki:bancho:multiplayer";
const SESSIONS_MATCHES_KEY: &str = "akatsuki:bancho:sessions:multiplayer";
pub const MULTIPLAYER_MAX_SIZE: usize = 16;
fn make_slots_key(match_id: i64) -> String {
    format!("akatsuki:bancho:multiplayer:{match_id}")
}

pub async fn create<C: Context>(
    ctx: &C,
    host_session_id: Uuid,
    host_user_id: i64,
    name: &str,
    password: &str,
    beatmap_name: &str,
    beatmap_md5: &str,
    beatmap_id: i32,
    mode: u8,
    max_player_count: usize,
) -> anyhow::Result<MultiplayerMatch> {
    let mut mp_match = MultiplayerMatch {
        beatmap_id,
        host_user_id,
        mode,
        name: name.to_string(),
        password: password.to_string(),
        beatmap_name: beatmap_name.to_string(),
        beatmap_md5: beatmap_md5.to_string(),
        ..Default::default()
    };
    let private = !password.is_empty();
    let query_result = sqlx::query("INSERT INTO matches (name, private) VALUES (?, ?)")
        .bind(name)
        .bind(private)
        .execute(ctx.db())
        .await?;
    mp_match.match_id = query_result.last_insert_id() as _;

    let slots: [(usize, Json<MultiplayerMatchSlot>); MULTIPLAYER_MAX_SIZE] =
        std::array::from_fn(|slot_id| {
            let mut slot = MultiplayerMatchSlot::default();
            match slot_id {
                // Place the host into the first slot
                0 => {
                    slot.status = SlotStatus::NotReady.bits();
                    slot.user_id = Some(host_user_id as _);
                }
                i if i >= max_player_count => slot.status = SlotStatus::Locked.bits(),
                _ => slot.status = SlotStatus::Empty.bits(),
            }
            (slot_id, Json(slot))
        });

    let mut redis = ctx.redis().await?;
    let slots_key = make_slots_key(mp_match.match_id);
    redis::pipe()
        .hset(SESSIONS_MATCHES_KEY, host_session_id, mp_match.match_id)
        .ignore()
        .hset_multiple(slots_key, &slots)
        .ignore()
        .hset(KEY, mp_match.match_id, Json(&mp_match))
        .ignore()
        .exec_async(redis.deref_mut())
        .await?;
    Ok(mp_match)
}

pub async fn fetch_session_match_id<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> anyhow::Result<Option<i64>> {
    let mut redis = ctx.redis().await?;
    Ok(redis.hget(SESSIONS_MATCHES_KEY, session_id).await?)
}

pub async fn fetch_all<C: Context>(
    ctx: &C,
) -> anyhow::Result<impl Iterator<Item = MultiplayerMatch>> {
    let mut redis = ctx.redis().await?;
    let matches: Vec<Json<MultiplayerMatch>> = redis.hvals(KEY).await?;
    Ok(matches.into_iter().map(Json::into_inner))
}

pub async fn fetch_slot<C: Context>(
    ctx: &C,
    match_id: i64,
    slot_id: usize,
) -> anyhow::Result<Option<MultiplayerMatchSlot>> {
    let mut redis = ctx.redis().await?;
    let slots_key = make_slots_key(match_id);
    let slot: Option<Json<MultiplayerMatchSlot>> = redis.hget(slots_key, slot_id).await?;
    Ok(slot.map(Json::into_inner))
}

const SLOT_IDS: [u8; MULTIPLAYER_MAX_SIZE] = {
    let mut out = [0; MULTIPLAYER_MAX_SIZE];
    let mut i = 0;
    while i < MULTIPLAYER_MAX_SIZE {
        out[i] = i as u8;
        i += 1;
    }
    out
};

pub async fn fetch_all_slots<C: Context>(
    ctx: &C,
    match_id: i64,
) -> anyhow::Result<[MultiplayerMatchSlot; MULTIPLAYER_MAX_SIZE]> {
    let mut redis = ctx.redis().await?;
    let slots_key = make_slots_key(match_id);
    // using HMGET instead of HGETALL guarantees order
    let slots: [Json<MultiplayerMatchSlot>; MULTIPLAYER_MAX_SIZE] =
        redis.hget(slots_key, &SLOT_IDS).await?;
    Ok(slots_from_json(slots))
}

// utility

fn slots_from_json(
    json: [Json<MultiplayerMatchSlot>; MULTIPLAYER_MAX_SIZE],
) -> [MultiplayerMatchSlot; MULTIPLAYER_MAX_SIZE] {
    std::array::from_fn(|i| json[i].0)
}
