use crate::common::context::Context;

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
