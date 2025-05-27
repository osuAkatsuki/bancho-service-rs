use crate::api::RequestContext;
use crate::entities::stats::Stats;
use crate::models::Gamemode;
use redis::AsyncCommands;

const TABLE_NAME: &str = "user_stats";
const READ_FIELDS: &str = r#"
user_id, mode, ranked_score, total_score, playcount, replays_watched,
total_hits, level, avg_accuracy, pp, playtime, xh_count, x_count, sh_count,
s_count, a_count, b_count, c_count, d_count, max_combo, latest_pp_awarded"#;

pub async fn fetch_one(ctx: &RequestContext, user_id: i64, mode: i16) -> sqlx::Result<Stats> {
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
        .fetch_one(&ctx.db)
        .await
}

pub async fn fetch_global_rank(
    ctx: &RequestContext,
    user_id: i64,
    mode: Gamemode,
) -> anyhow::Result<Option<usize>> {
    const BOARDS: [&'static str; 3] = ["leaderboard", "relaxboard", "autoboard"];
    const MODES: [&'static str; 4] = ["std", "taiko", "ctb", "mania"];
    let board = BOARDS[mode.rxap() as usize];
    let mode = MODES[mode.to_bancho() as usize];
    let key = format!("ripple:{board}:{mode}");
    let mut redis = ctx.redis.get().await?;
    Ok(redis.zrevrank(key, user_id).await?)
}
