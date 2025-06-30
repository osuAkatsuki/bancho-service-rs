use crate::common::context::Context;
use crate::common::redis_json::Json;
use crate::entities::multiplayer::{MultiplayerMatch, MultiplayerMatchSlot};
use crate::entities::sessions::SessionIdentity;
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
    host_identity: SessionIdentity,
    name: &str,
    password: &str,
    beatmap_name: &str,
    beatmap_md5: &str,
    beatmap_id: i32,
    mode: u8,
    max_player_count: usize,
) -> anyhow::Result<(
    MultiplayerMatch,
    [MultiplayerMatchSlot; MULTIPLAYER_MAX_SIZE],
)> {
    let mut mp_match = MultiplayerMatch {
        beatmap_id,
        mode,
        name: name.to_string(),
        password: password.to_string(),
        host_user_id: host_identity.user_id,
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
                0 => slot.prepare(host_identity),
                i if i >= max_player_count => slot.status = SlotStatus::Locked.bits(),
                _ => slot.status = SlotStatus::Empty.bits(),
            }
            (slot_id, Json(slot))
        });

    let mut redis = ctx.redis().await?;
    let slots_key = make_slots_key(mp_match.match_id);
    redis::pipe()
        .atomic()
        .hset(
            SESSIONS_MATCHES_KEY,
            host_identity.session_id,
            mp_match.match_id,
        )
        .ignore()
        .hset_multiple(slots_key, &slots)
        .ignore()
        .hset(KEY, mp_match.match_id, Json(&mp_match))
        .ignore()
        .exec_async(redis.deref_mut())
        .await?;
    Ok((mp_match, slots_from_json_with_index(slots)))
}

pub async fn delete<C: Context>(ctx: &C, match_id: i64) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let slots_key = make_slots_key(match_id);
    redis::pipe()
        .atomic()
        .del(slots_key)
        .ignore()
        .hdel(KEY, match_id)
        .ignore()
        .exec_async(redis.deref_mut())
        .await?;

    sqlx::query("UPDATE matches SET end_time = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(match_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn join<C: Context>(
    ctx: &C,
    identity: SessionIdentity,
    match_id: i64,
) -> anyhow::Result<Option<[MultiplayerMatchSlot; MULTIPLAYER_MAX_SIZE]>> {
    let mut slots = fetch_all_slots(ctx, match_id).await?;
    let (slot_id, slot) = match slots
        .iter_mut()
        .enumerate()
        .find(|(_, slot)| slot.status == SlotStatus::Empty.bits())
    {
        Some((id, slot)) => {
            slot.prepare(identity);
            (id, *slot)
        }
        None => return Ok(None),
    };

    let mut redis = ctx.redis().await?;
    let slots_key = make_slots_key(match_id);
    redis::pipe()
        .atomic()
        .hset(SESSIONS_MATCHES_KEY, identity.session_id, match_id)
        .ignore()
        .hset(slots_key, slot_id, Json(slot))
        .ignore()
        .exec_async(redis.deref_mut())
        .await?;
    Ok(Some(slots))
}

pub async fn leave<C: Context>(
    ctx: &C,
    session_id: Uuid,
    user_id: i64,
    match_id: i64,
) -> anyhow::Result<Option<(usize, [MultiplayerMatchSlot; MULTIPLAYER_MAX_SIZE])>> {
    let mut slots = fetch_all_slots(ctx, match_id).await?;
    let (slot_id, slot) = match slots.iter_mut().enumerate().find(|(_, slot)| {
        slot.user
            .is_some_and(|slot_user| slot_user.user_id == user_id)
    }) {
        Some((id, slot)) => {
            slot.clear();
            (id, *slot)
        }
        None => return Ok(None),
    };
    let user_count = slots.iter().filter(|slot| slot.user.is_some()).count();

    let slots_key = make_slots_key(match_id);
    let mut pipe = redis::pipe();
    pipe.atomic()
        .hdel(SESSIONS_MATCHES_KEY, session_id)
        .ignore();
    if user_count == 0 {
        pipe.hdel(KEY, match_id).ignore().del(slots_key).ignore();
    } else {
        pipe.hset(slots_key, slot_id, Json(slot)).ignore();
    }
    let mut redis = ctx.redis().await?;
    pipe.exec_async(redis.deref_mut()).await?;
    Ok(Some((user_count, slots)))
}

pub async fn fetch_session_match_id<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> anyhow::Result<Option<i64>> {
    let mut redis = ctx.redis().await?;
    Ok(redis.hget(SESSIONS_MATCHES_KEY, session_id).await?)
}

pub async fn fetch_one<C: Context>(
    ctx: &C,
    match_id: i64,
) -> anyhow::Result<Option<MultiplayerMatch>> {
    let mut redis = ctx.redis().await?;
    let mp_match: Option<Json<MultiplayerMatch>> = redis.hget(KEY, match_id).await?;
    Ok(mp_match.map(Json::into_inner))
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

const SLOT_IDS: [usize; MULTIPLAYER_MAX_SIZE] = {
    let mut out = [0; MULTIPLAYER_MAX_SIZE];
    let mut i = 0;
    while i < MULTIPLAYER_MAX_SIZE {
        out[i] = i;
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

pub async fn update<C: Context>(
    ctx: &C,
    mp_match: MultiplayerMatch,
    update_persistent: bool,
) -> anyhow::Result<MultiplayerMatch> {
    let mut redis = ctx.redis().await?;
    let _: () = redis.hset(KEY, mp_match.match_id, Json(&mp_match)).await?;

    if update_persistent {
        let is_private = !mp_match.password.is_empty();
        sqlx::query("UPDATE matches SET name = ?, private = ? WHERE id = ?")
            .bind(&mp_match.name)
            .bind(is_private)
            .bind(mp_match.match_id)
            .execute(ctx.db())
            .await?;
    }

    Ok(mp_match)
}

pub async fn update_slot<C: Context>(
    ctx: &C,
    match_id: i64,
    slot_id: usize,
    slot: MultiplayerMatchSlot,
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let slots_key = make_slots_key(match_id);
    let _: () = redis.hset(slots_key, slot_id, Json(&slot)).await?;
    Ok(())
}

pub async fn update_slots<const N: usize, C: Context>(
    ctx: &C,
    match_id: i64,
    slots: [(usize, MultiplayerMatchSlot); N],
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let slots_key = make_slots_key(match_id);
    let slots: [(usize, Json<MultiplayerMatchSlot>); N] =
        std::array::from_fn(|i| (slots[i].0, Json(slots[i].1)));
    let _: () = redis.hset_multiple(slots_key, &slots).await?;
    Ok(())
}

pub async fn update_all_slots<C: Context>(
    ctx: &C,
    match_id: i64,
    slots: [MultiplayerMatchSlot; MULTIPLAYER_MAX_SIZE],
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let slots_key = make_slots_key(match_id);
    let slots: [_; MULTIPLAYER_MAX_SIZE] = std::array::from_fn(|i| (i, Json(slots[i])));
    let _: () = redis.hset_multiple(slots_key, &slots).await?;
    Ok(())
}

// utility

fn slots_from_json(
    json: [Json<MultiplayerMatchSlot>; MULTIPLAYER_MAX_SIZE],
) -> [MultiplayerMatchSlot; MULTIPLAYER_MAX_SIZE] {
    std::array::from_fn(|i| json[i].0)
}

/// NOTE: this requires the json array to be ordered correctly (0, 1, 2, ...)
fn slots_from_json_with_index(
    json: [(usize, Json<MultiplayerMatchSlot>); MULTIPLAYER_MAX_SIZE],
) -> [MultiplayerMatchSlot; MULTIPLAYER_MAX_SIZE] {
    std::array::from_fn(|i| json[i].1.0)
}
