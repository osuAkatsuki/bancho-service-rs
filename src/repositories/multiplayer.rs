use crate::common::context::{Context, PoolContext};
use crate::common::redis_json::Json;
use crate::entities::multiplayer::{MultiplayerMatch, MultiplayerMatchSlot};
use crate::entities::sessions::SessionIdentity;
use bancho_protocol::structures::SlotStatus;
use redis::AsyncCommands;
use std::ops::DerefMut;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerType {
    Regular,
    MatchStart,
}

const KEY: &str = "akatsuki:bancho:multiplayer";
const SESSIONS_MATCHES_KEY: &str = "akatsuki:bancho:sessions:multiplayer";
// Active matches keep their durable database ID internally and borrow a u16 ID for the osu! wire protocol.
const WIRE_MATCHES_KEY: &str = "akatsuki:bancho:multiplayer:wire_matches";
const MATCH_WIRE_IDS_KEY: &str = "akatsuki:bancho:multiplayer:match_wire_ids";
const NEXT_WIRE_ID_KEY: &str = "akatsuki:bancho:multiplayer:next_wire_id";
const WIRE_ID_CAPACITY: u32 = u16::MAX as u32 + 1;
pub const MULTIPLAYER_MAX_SIZE: usize = 16;

const INITIALIZE_WIRE_IDS_SCRIPT: &str = r#"
local capacity = tonumber(ARGV[1])
local active_match_ids = redis.call('HKEYS', KEYS[1])
local active_matches = {}

table.sort(active_match_ids, function(a, b)
    return tonumber(a) < tonumber(b)
end)

for _, match_id in ipairs(active_match_ids) do
    active_matches[match_id] = true
end

local wire_mappings = redis.call('HGETALL', KEYS[2])
for i = 1, #wire_mappings, 2 do
    local wire_id = wire_mappings[i]
    local match_id = wire_mappings[i + 1]
    local reverse_wire_id = redis.call('HGET', KEYS[3], match_id)
    if not active_matches[match_id]
        or reverse_wire_id == false
        or tostring(reverse_wire_id) ~= tostring(wire_id) then
        redis.call('HDEL', KEYS[2], wire_id)
    end
end

local reverse_mappings = redis.call('HGETALL', KEYS[3])
for i = 1, #reverse_mappings, 2 do
    local match_id = reverse_mappings[i]
    local wire_id = reverse_mappings[i + 1]
    local mapped_match_id = redis.call('HGET', KEYS[2], wire_id)
    if not active_matches[match_id]
        or mapped_match_id == false
        or tostring(mapped_match_id) ~= tostring(match_id) then
        redis.call('HDEL', KEYS[3], match_id)
    end
end

local function find_available_wire_id(preferred_wire_id)
    if redis.call('HEXISTS', KEYS[2], preferred_wire_id) == 0 then
        return preferred_wire_id
    end

    local cursor = tonumber(redis.call('GET', KEYS[4]) or '0')
    for offset = 0, capacity - 1 do
        local candidate = (cursor + offset) % capacity
        if redis.call('HEXISTS', KEYS[2], candidate) == 0 then
            return candidate
        end
    end

    return nil
end

for _, match_id in ipairs(active_match_ids) do
    local match_json = redis.call('HGET', KEYS[1], match_id)
    local match_data = cjson.decode(match_json)
    local wire_id = redis.call('HGET', KEYS[3], match_id)

    if wire_id == false then
        local preferred_wire_id
        if type(match_data.wire_id) == 'number'
            and match_data.wire_id >= 0
            and match_data.wire_id < capacity
            and match_data.wire_id == math.floor(match_data.wire_id) then
            preferred_wire_id = match_data.wire_id
        else
            preferred_wire_id = tonumber(match_id) % capacity
        end

        wire_id = find_available_wire_id(preferred_wire_id)
        if wire_id == nil then
            return redis.error_reply('no multiplayer wire IDs available')
        end

        redis.call('HSET', KEYS[2], wire_id, match_id)
        redis.call('HSET', KEYS[3], match_id, wire_id)
        redis.call('SET', KEYS[4], (wire_id + 1) % capacity)
    else
        wire_id = tonumber(wire_id)
    end

    match_data.wire_id = wire_id
    redis.call('HSET', KEYS[1], match_id, cjson.encode(match_data))
end

return #active_match_ids
"#;

const CREATE_MATCH_SCRIPT: &str = r#"
local capacity = tonumber(ARGV[1])
local wire_id = tonumber(ARGV[2])
local match_id = ARGV[3]

if redis.call('HEXISTS', KEYS[3], wire_id) == 1 then
    return 0
end

local match_data = cjson.decode(ARGV[4])
match_data.wire_id = wire_id

redis.call('HSET', KEYS[1], match_id, cjson.encode(match_data))
redis.call('HSET', KEYS[2], ARGV[5], match_id)
redis.call('HSET', KEYS[3], wire_id, match_id)
redis.call('HSET', KEYS[4], match_id, wire_id)
redis.call('SET', KEYS[5], (wire_id + 1) % capacity)

local slot_count = tonumber(ARGV[6])
for slot_id = 0, slot_count - 1 do
    redis.call('HSET', KEYS[6], slot_id, ARGV[7 + slot_id])
end

return 1
"#;

const DELETE_MATCH_SCRIPT: &str = r#"
local match_id = ARGV[1]
local wire_id = redis.call('HGET', KEYS[1], match_id)

redis.call('HDEL', KEYS[2], match_id)
redis.call('DEL', KEYS[3], KEYS[4], KEYS[5], KEYS[6])

if wire_id == false then
    return -1
end
return tonumber(wire_id)
"#;

const RELEASE_WIRE_ID_SCRIPT: &str = r#"
local match_id = ARGV[1]
local wire_id = redis.call('HGET', KEYS[2], match_id)

if wire_id == false then
    return 0
end

local mapped_match_id = redis.call('HGET', KEYS[1], wire_id)
if mapped_match_id ~= false and tostring(mapped_match_id) == tostring(match_id) then
    redis.call('HDEL', KEYS[1], wire_id)
end
redis.call('HDEL', KEYS[2], match_id)
return 1
"#;

fn wire_id_candidates(start: u16) -> impl Iterator<Item = u16> {
    (0..WIRE_ID_CAPACITY).map(move |offset| start.wrapping_add(offset as u16))
}

pub async fn initialize_wire_ids<C: Context>(ctx: &C) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let _: usize = redis::Script::new(INITIALIZE_WIRE_IDS_SCRIPT)
        .key(KEY)
        .key(WIRE_MATCHES_KEY)
        .key(MATCH_WIRE_IDS_KEY)
        .key(NEXT_WIRE_ID_KEY)
        .arg(WIRE_ID_CAPACITY)
        .invoke_async(redis.deref_mut())
        .await?;
    Ok(())
}

fn make_referees_key(match_id: i64) -> String {
    format!("akatsuki:bancho:multiplayer:referees:{match_id}")
}

fn make_slots_key(match_id: i64) -> String {
    format!("akatsuki:bancho:multiplayer:{match_id}")
}

fn make_timer_key(match_id: i64, timer_type: TimerType) -> String {
    match timer_type {
        TimerType::Regular => format!("akatsuki:bancho:multiplayer:timer:{match_id}"),
        TimerType::MatchStart => format!("akatsuki:bancho:multiplayer:start_timer:{match_id}"),
    }
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

    let slots_key = make_slots_key(mp_match.match_id);
    let mut redis = ctx.redis().await?;
    let next_wire_id: Option<u16> = redis.get(NEXT_WIRE_ID_KEY).await?;

    for wire_id in wire_id_candidates(next_wire_id.unwrap_or_default()) {
        let script = redis::Script::new(CREATE_MATCH_SCRIPT);
        let mut invocation = script.prepare_invoke();
        invocation
            .key(KEY)
            .key(SESSIONS_MATCHES_KEY)
            .key(WIRE_MATCHES_KEY)
            .key(MATCH_WIRE_IDS_KEY)
            .key(NEXT_WIRE_ID_KEY)
            .key(&slots_key)
            .arg(WIRE_ID_CAPACITY)
            .arg(wire_id)
            .arg(mp_match.match_id)
            .arg(Json(&mp_match))
            .arg(host_identity.session_id)
            .arg(MULTIPLAYER_MAX_SIZE);
        for (_, slot) in &slots {
            invocation.arg(slot);
        }

        let created: i32 = invocation.invoke_async(redis.deref_mut()).await?;
        if created == 1 {
            mp_match.wire_id = Some(wire_id);
            return Ok((mp_match, slots_from_json_with_index(slots)));
        }
    }

    anyhow::bail!("all multiplayer wire IDs are in use")
}

pub async fn delete<C: Context>(ctx: &C, match_id: i64) -> anyhow::Result<Option<u16>> {
    let mut redis = ctx.redis().await?;
    let slots_key = make_slots_key(match_id);
    let referees_key = make_referees_key(match_id);
    let timer_key = make_timer_key(match_id, TimerType::Regular);
    let start_timer_key = make_timer_key(match_id, TimerType::MatchStart);
    let wire_id: i64 = redis::Script::new(DELETE_MATCH_SCRIPT)
        .key(MATCH_WIRE_IDS_KEY)
        .key(KEY)
        .key(slots_key)
        .key(referees_key)
        .key(timer_key)
        .key(start_timer_key)
        .arg(match_id)
        .invoke_async(redis.deref_mut())
        .await?;

    sqlx::query("UPDATE matches SET end_time = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(match_id)
        .execute(ctx.db())
        .await?;
    Ok(u16::try_from(wire_id).ok())
}

pub async fn release_wire_id<C: Context>(ctx: &C, match_id: i64) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let _: i32 = redis::Script::new(RELEASE_WIRE_ID_SCRIPT)
        .key(WIRE_MATCHES_KEY)
        .key(MATCH_WIRE_IDS_KEY)
        .arg(match_id)
        .invoke_async(redis.deref_mut())
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
    match_id: i64,
) -> anyhow::Result<Option<(usize, [MultiplayerMatchSlot; MULTIPLAYER_MAX_SIZE])>> {
    let mut slots = fetch_all_slots(ctx, match_id).await?;
    let (slot_id, slot) = match slots.iter_mut().enumerate().find(|(_, slot)| {
        slot.user
            .is_some_and(|slot_user| slot_user.session_id == session_id)
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

pub async fn fetch_by_wire_id<C: Context>(
    ctx: &C,
    wire_id: u16,
) -> anyhow::Result<Option<MultiplayerMatch>> {
    let mut redis = ctx.redis().await?;
    let match_id: Option<i64> = redis.hget(WIRE_MATCHES_KEY, wire_id).await?;
    let Some(match_id) = match_id else {
        return Ok(None);
    };
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

// Referees

pub async fn add_referee<C: Context>(ctx: &C, match_id: i64, user_id: i64) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let referees_key = make_referees_key(match_id);
    let _: () = redis.sadd(referees_key, user_id).await?;
    Ok(())
}

pub async fn remove_referee<C: Context>(
    ctx: &C,
    match_id: i64,
    user_id: i64,
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let referees_key = make_referees_key(match_id);
    let _: () = redis.srem(referees_key, user_id).await?;
    Ok(())
}

pub async fn get_referees<C: Context>(ctx: &C, match_id: i64) -> anyhow::Result<Vec<i64>> {
    let mut redis = ctx.redis().await?;
    let referees_key = make_referees_key(match_id);
    let referees = redis.smembers(referees_key).await?;
    Ok(referees)
}

pub async fn is_referee<C: Context>(ctx: &C, match_id: i64, user_id: i64) -> anyhow::Result<bool> {
    let mut redis = ctx.redis().await?;
    let referees_key = make_referees_key(match_id);
    let is_referee = redis.sismember(referees_key, user_id).await?;
    Ok(is_referee)
}

pub async fn clear_referees<C: Context>(ctx: &C, match_id: i64) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let referees_key = make_referees_key(match_id);
    let _: () = redis.del(referees_key).await?;
    Ok(())
}

// Timers

pub async fn set_timer<C: Context>(
    ctx: &C,
    match_id: i64,
    timer_type: TimerType,
    seconds: u64,
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let timer_key = make_timer_key(match_id, timer_type);
    let _: () = redis.set(timer_key, seconds).await?;
    Ok(())
}

pub async fn get_timer<C: Context>(
    ctx: &C,
    match_id: i64,
    timer_type: TimerType,
) -> anyhow::Result<Option<i64>> {
    let mut redis = ctx.redis().await?;
    let timer_key = make_timer_key(match_id, timer_type);
    let remaining_seconds = redis.get(timer_key).await?;
    Ok(remaining_seconds)
}

pub async fn decrease_timer<C: Context>(
    ctx: &C,
    match_id: i64,
    timer_type: TimerType,
) -> anyhow::Result<i64> {
    let mut redis = ctx.redis().await?;
    let timer_key = make_timer_key(match_id, timer_type);
    let remaining_seconds = redis.decr(timer_key, 1).await?;
    Ok(remaining_seconds)
}

pub async fn abort_timer<C: Context>(
    ctx: &C,
    match_id: i64,
    timer_type: TimerType,
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let timer_key = make_timer_key(match_id, timer_type);
    let _: () = redis.del(timer_key).await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn wire_id_candidates_wrap_at_u16_boundary() {
        let candidates: Vec<_> = wire_id_candidates(u16::MAX - 1).take(4).collect();

        assert_eq!(candidates, [u16::MAX - 1, u16::MAX, 0, 1]);
    }

    #[test]
    fn wire_id_candidates_visit_every_reusable_id_once() {
        let candidates: Vec<_> = wire_id_candidates(42).collect();
        let unique: HashSet<_> = candidates.iter().copied().collect();

        assert_eq!(candidates.len(), WIRE_ID_CAPACITY as usize);
        assert_eq!(unique.len(), WIRE_ID_CAPACITY as usize);
        assert_eq!(candidates.first(), Some(&42));
        assert_eq!(candidates.last(), Some(&41));
    }
}
