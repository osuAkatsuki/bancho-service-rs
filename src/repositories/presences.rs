use crate::common::context::Context;
use crate::common::redis_json::Json;
use crate::entities::bot;
use crate::entities::presences::Presence;
use redis::AsyncCommands;
use std::ops::DerefMut;
use tracing::warn;

const KEY: &'static str = "akatsuki:bancho:presences";

pub async fn create<C: Context>(
    ctx: &C,
    user_id: i64,
    username: String,
    privileges: u8,
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
        username,
        privileges,
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
    let mut redis = ctx.redis().await?;
    let _: () = redis.hset(KEY, user_id, Json(&presence)).await?;
    Ok(presence)
}

pub async fn fetch_one<C: Context>(ctx: &C, user_id: i64) -> anyhow::Result<Option<Presence>> {
    if user_id == bot::BOT_ID {
        return Ok(Some(bot::presence()));
    }

    let mut redis = ctx.redis().await?;
    let presence: Option<Json<Presence>> = redis.hget(KEY, user_id).await?;
    Ok(presence.map(Json::into_inner))
}

pub async fn fetch_multiple<C: Context>(
    ctx: &C,
    user_ids: &[i32],
) -> anyhow::Result<impl Iterator<Item = Option<Presence>>> {
    let mut redis = ctx.redis().await?;
    let presences: Vec<Option<Json<Presence>>> = redis::cmd("HMGET")
        .arg(KEY)
        .arg(user_ids)
        .query_async(redis.deref_mut())
        .await?;
    let presences = presences.into_iter().enumerate().map(|(i, presence)| {
        let user_id = user_ids[i];
        if user_id == (bot::BOT_ID as i32) {
            return Some(bot::presence());
        } else {
            presence.map(Json::into_inner)
        }
    });
    Ok(presences)
}

pub async fn fetch_user_ids<C: Context>(ctx: &C) -> anyhow::Result<Vec<i32>> {
    let mut redis = ctx.redis().await?;
    let mut user_ids: Vec<i32> = redis.hkeys(KEY).await?;
    user_ids.push(bot::BOT_ID as _);
    Ok(user_ids)
}

pub async fn fetch_all<C: Context>(ctx: &C) -> anyhow::Result<impl Iterator<Item = Presence>> {
    let mut redis = ctx.redis().await?;
    let mut presences: Vec<Json<Presence>> = redis.hvals(KEY).await?;
    presences.push(Json(bot::presence()));
    Ok(presences.into_iter().map(Json::into_inner))
}

pub async fn update<C: Context>(ctx: &C, presence: Presence) -> anyhow::Result<Presence> {
    if presence.user_id == bot::BOT_ID {
        return Ok(bot::presence());
    }

    let mut redis = ctx.redis().await?;
    let _: () = redis.hset(KEY, presence.user_id, Json(&presence)).await?;
    Ok(presence)
}

pub async fn delete<C: Context>(ctx: &C, user_id: i64) -> anyhow::Result<()> {
    if user_id == bot::BOT_ID {
        warn!("Tried to delete bot presence, ignoring.");
        return Ok(());
    }

    let mut redis = ctx.redis().await?;
    Ok(redis.hdel(KEY, user_id).await?)
}
