use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::entities::gamemodes::{CustomGamemode, Gamemode};
use crate::models::stats::Stats;
use crate::repositories::stats;
use bancho_protocol::structures::{Country, Mode};

pub async fn fetch_one<C: Context>(ctx: &C, user_id: i64, mode: Gamemode) -> ServiceResult<Stats> {
    match stats::fetch_one(ctx, user_id, mode as _).await {
        Ok(stats) => Ok(Stats::from(stats)),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_global_rank<C: Context>(
    ctx: &C,
    user_id: i64,
    mode: Gamemode,
) -> ServiceResult<usize> {
    match stats::fetch_global_rank(ctx, user_id, mode).await {
        Ok(Some(rank)) => Ok(rank + 1),
        Ok(None) => Ok(0),
        Err(e) => unexpected(e),
    }
}

pub async fn remove_from_leaderboard<C: Context>(
    ctx: &C,
    user_id: i64,
    country: Country,
    mode: Option<Mode>,
    custom_gamemode: Option<CustomGamemode>,
) -> ServiceResult<()> {
    match stats::remove_from_leaderboard(ctx, user_id, country, mode, custom_gamemode).await {
        Ok(()) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn add_to_leaderboards<C: Context>(
    ctx: &C,
    user_id: i64,
    country: Country,
) -> ServiceResult<()> {
    let stats = stats::fetch_user_stats(ctx, user_id).await?;
    for mode_stats in stats {
        let gamemode = Gamemode::from_value(mode_stats.mode);
        if mode_stats.pp != 0 {
            stats::add_to_leaderboard(ctx, user_id, country, gamemode, mode_stats.pp).await?;
        }
    }
    Ok(())
}
