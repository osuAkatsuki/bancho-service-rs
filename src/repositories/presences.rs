use crate::api::RequestContext;
use crate::common::redis_json::Json;
use crate::entities::presences::Presence;
use redis::AsyncCommands;
use std::ops::DerefMut;

const KEY: &'static str = "akatsuki:bancho:presences";

pub async fn create(
    ctx: &RequestContext,
    user_id: i64,
    action: u8,
    info_text: String,
    beatmap_md5: String,
    beatmap_id: i32,
    mods: u32,
    mode: u8,
    ranked_score: u64,
    total_score: u64,
    accuracy: f64,
    playcount: u32,
    performance: u32,
    global_rank: usize,
    country_code: String,
    latitude: f32,
    longitude: f32,
    utc_offset: i8,
) -> anyhow::Result<Presence> {
    let presence = Presence {
        user_id,
        action,
        info_text,
        beatmap_md5,
        beatmap_id,
        mods,
        mode,
        ranked_score,
        total_score,
        accuracy,
        playcount,
        performance,
        global_rank,
        country_code,
        latitude,
        longitude,
        utc_offset,
    };
    let mut redis = ctx.redis.get().await?;
    let _: () = redis.hset(KEY, user_id, Json(&presence)).await?;
    Ok(presence)
}

pub async fn fetch_one(ctx: &RequestContext, user_id: i64) -> anyhow::Result<Option<Presence>> {
    let mut redis = ctx.redis.get().await?;
    let presence: Option<Json<Presence>> = redis.hget(KEY, user_id).await?;
    Ok(presence.map(Json::into_inner))
}

pub async fn fetch_multiple(
    ctx: &RequestContext,
    user_ids: &[i32],
) -> anyhow::Result<impl Iterator<Item = Option<Presence>>> {
    let mut redis = ctx.redis.get().await?;
    let presences: Vec<Option<Json<Presence>>> = redis::cmd("HMGET")
        .arg(KEY)
        .arg(user_ids)
        .query_async(redis.deref_mut())
        .await?;
    let presences = presences
        .into_iter()
        .map(|presence| presence.map(Json::into_inner));
    Ok(presences)
}

pub async fn fetch_user_ids(ctx: &RequestContext) -> anyhow::Result<Vec<i32>> {
    let mut redis = ctx.redis.get().await?;
    let user_ids: Vec<i32> = redis.hkeys(KEY).await?;
    Ok(user_ids)
}

pub async fn fetch_all(ctx: &RequestContext) -> anyhow::Result<impl Iterator<Item = Presence>> {
    let mut redis = ctx.redis.get().await?;
    let presences: Vec<Json<Presence>> = redis.hvals(KEY).await?;
    Ok(presences.into_iter().map(Json::into_inner))
}

pub async fn update(ctx: &RequestContext, presence: Presence) -> anyhow::Result<Presence> {
    let mut redis = ctx.redis.get().await?;
    let _: () = redis.hset(KEY, presence.user_id, Json(&presence)).await?;
    Ok(presence)
}

pub async fn delete(ctx: &RequestContext, user_id: i64) -> anyhow::Result<()> {
    let mut redis = ctx.redis.get().await?;
    Ok(redis.hdel(KEY, user_id).await?)
}
