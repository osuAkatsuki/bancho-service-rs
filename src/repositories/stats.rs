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

const BOARDS: [&'static str; 3] = ["leaderboard", "relaxboard", "autoboard"];

const MODES_STR: [&'static str; 4] = ["std", "taiko", "ctb", "mania"];
const MODES: [Mode; 4] = [Mode::Standard, Mode::Taiko, Mode::Catch, Mode::Mania];
const CUSTOM_GAMEMODES: [CustomGamemode; 3] = [
    CustomGamemode::Vanilla,
    CustomGamemode::Relax,
    CustomGamemode::Autopilot,
];

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
    mode: Option<Mode>,
    custom_gamemode: Option<CustomGamemode>,
) -> anyhow::Result<()> {
    let boards = match custom_gamemode {
        Some(relax) => &[relax],
        None => &CUSTOM_GAMEMODES[..],
    };
    let modes = match mode {
        None => &MODES[..],
        Some(mode) => &[mode],
    };

    let mut redis = ctx.redis().await?;
    let mut pipe = redis::pipe();
    let country_code = user_country.code();
    for board in boards {
        for mode in modes {
            let key = make_key(*mode, *board);
            let country_key = make_country_key(*mode, *board, country_code);
            pipe.zrem(key, user_id)
                .ignore()
                .zrem(country_key, user_id)
                .ignore();
        }
    }

    pipe.exec_async(redis.deref_mut()).await?;
    Ok(())
}
