use crate::common::context::Context;
use crate::entities::gamemodes::{CustomGamemode, Gamemode};
use crate::entities::stats::Stats;
use bancho_protocol::structures::{Country, Mode};
use redis::AsyncCommands;
use std::ops::DerefMut;

const TABLE_NAME: &str = "user_stats";
const READ_FIELDS: &str = r#"
user_id, mode, ranked_score, total_score, playcount, replays_watched,
total_hits, level, avg_accuracy, pp, playtime, xh_count, x_count, sh_count,
s_count, a_count, b_count, c_count, d_count, max_combo, latest_pp_awarded"#;

pub async fn fetch_one<C: Context>(ctx: &C, user_id: i64, mode: i16) -> sqlx::Result<Stats> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " WHERE user_id = ? AND mode = ?"
    );
    sqlx::query_as(QUERY)
        .bind(user_id)
        .bind(mode)
        .fetch_one(ctx.db())
        .await
}

pub async fn fetch_user_stats<C: Context>(ctx: &C, user_id: i64) -> sqlx::Result<Vec<Stats>> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " WHERE user_id = ?"
    );
    sqlx::query_as(QUERY)
        .bind(user_id)
        .fetch_all(ctx.db())
        .await
}

const BOARDS: [&'static str; 3] = ["leaderboard", "relaxboard", "autoboard"];
const MODES_STR: [&'static str; 4] = ["std", "taiko", "ctb", "mania"];

fn make_key(mode: Mode, custom_gamemode: CustomGamemode) -> String {
    let board = BOARDS[custom_gamemode as usize];
    let mode = MODES_STR[mode as usize];
    format!("ripple:{board}:{mode}")
}

fn make_country_key(mode: Mode, custom_gamemode: CustomGamemode, country: &str) -> String {
    let board = BOARDS[custom_gamemode as usize];
    let mode = MODES_STR[mode as usize];
    let country = country.to_lowercase();
    format!("ripple:{board}:{mode}:{country}")
}

pub async fn fetch_global_rank<C: Context>(
    ctx: &C,
    user_id: i64,
    mode: Gamemode,
) -> anyhow::Result<Option<usize>> {
    let key = make_key(mode.to_bancho(), mode.custom_gamemode());
    let mut redis = ctx.redis().await?;
    Ok(redis.zrevrank(key, user_id).await?)
}

pub async fn remove_from_leaderboard<C: Context>(
    ctx: &C,
    user_id: i64,
    user_country: Country,
    gamemode: Gamemode,
) -> anyhow::Result<()> {
    let mode = gamemode.to_bancho();
    let custom_mode = gamemode.custom_gamemode();
    let key = make_key(mode, custom_mode);
    let country_key = make_country_key(mode, custom_mode, user_country.code());

    let mut redis = ctx.redis().await?;
    redis::pipe()
        .zrem(key, user_id)
        .ignore()
        .zrem(country_key, user_id)
        .ignore()
        .exec_async(redis.deref_mut())
        .await?;
    Ok(())
}

pub async fn add_to_leaderboard<C: Context>(
    ctx: &C,
    user_id: i64,
    user_country: Country,
    gamemode: Gamemode,
    performance: u32,
) -> anyhow::Result<()> {
    let mode = gamemode.to_bancho();
    let custom_mode = gamemode.custom_gamemode();
    let key = make_key(mode, custom_mode);
    let country_key = make_country_key(mode, custom_mode, user_country.code());

    let mut redis = ctx.redis().await?;
    redis::pipe()
        .atomic()
        .zadd(key, user_id, performance)
        .ignore()
        .zadd(country_key, user_id, performance)
        .ignore()
        .exec_async(redis.deref_mut())
        .await?;
    Ok(())
}
