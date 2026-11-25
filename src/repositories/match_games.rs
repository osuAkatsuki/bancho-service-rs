use crate::common::context::{Context, PoolContext};

pub async fn create<C: Context>(
    ctx: &C,
    match_id: i64,
    beatmap_id: i32,
    mode: u8,
    mods: u32,
    win_condition: u8,
    team_type: u8,
) -> sqlx::Result<i64> {
    const QUERY: &str = concat!(
        "INSERT INTO match_games ",
        "(match_id, beatmap_id, mode, mods, scoring_type, team_type) ",
        "VALUES (?, ?, ?, ?, ?, ?)",
    );
    let res = sqlx::query(QUERY)
        .bind(match_id)
        .bind(beatmap_id)
        .bind(mode)
        .bind(mods)
        .bind(win_condition)
        .bind(team_type)
        .execute(ctx.db())
        .await?;
    Ok(res.last_insert_id() as _)
}

pub async fn game_ended<C: Context>(ctx: &C, match_id: i64) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE match_games SET end_time = CURRENT_TIMESTAMP WHERE match_id = ? AND end_time IS NULL";
    sqlx::query(QUERY).bind(match_id).execute(ctx.db()).await?;
    Ok(())
}

pub async fn scores<C: Context>(
    ctx: &C,
    match_id: i64,
    game_id: i64,
) -> sqlx::Result<Vec<(String, i32, f32, bool)>> {
    sqlx::query_as(
        "SELECT users.username, scores.score, scores.accuracy, scores.passed
         FROM match_game_scores AS scores JOIN users ON users.id = scores.user_id
         WHERE scores.match_id = ? AND scores.game_id = ?
         ORDER BY scores.score DESC LIMIT 16",
    )
    .bind(match_id)
    .bind(game_id)
    .fetch_all(ctx.db())
    .await
}
