use crate::common::context::Context;
use crate::entities::gamemodes::{CustomGamemode, Gamemode};
use crate::entities::scores::{FirstPlaceScore, MinimalScore, NewFirstPlace};
use bancho_protocol::structures::Mode;
use sqlx::Arguments;
use sqlx::mysql::MySqlArguments;

const SCORES_TABLES: [&str; 3] = ["scores", "scores_relax", "scores_ap"];

pub async fn fetch_first_places<C: Context>(
    ctx: &C,
    user_id: i64,
    mode: Option<Mode>,
    custom_gamemode: Option<CustomGamemode>,
) -> anyhow::Result<Vec<FirstPlaceScore>> {
    let mut query =
        String::from("SELECT scoreid, beatmap_md5, mode, rx FROM scores_first WHERE userid = ?");
    let mut args = MySqlArguments::default();
    args.add(user_id)
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    if let Some(mode) = mode {
        query.push_str(" AND mode = ?");
        args.add(mode as u8)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }
    if let Some(gamemode) = custom_gamemode {
        query.push_str(" AND rx = ?");
        args.add(gamemode as u8)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }

    let first_places: Vec<FirstPlaceScore> = sqlx::query_as_with(&query, args)
        .fetch_all(ctx.db())
        .await?;
    Ok(first_places)
}

// TODO: move to usecases
pub async fn remove_first_places<C: Context>(
    ctx: &C,
    user_id: i64,
    mode: Option<Mode>,
    custom_gamemode: Option<CustomGamemode>,
) -> anyhow::Result<()> {
    let first_places = fetch_first_places(ctx, user_id, mode, custom_gamemode).await?;
    for first_place in first_places {
        if first_place.mode > 3 || first_place.rx > 2 {
            continue;
        }
        let sort = match first_place.rx {
            0 => "score",
            _ => "pp",
        };
        let table = SCORES_TABLES[first_place.rx as usize];
        let query = format!(
            r#"
                SELECT s.id, s.userid FROM {table} s
                INNER JOIN {table} users u ON s.userid = u.id
                WHERE s.beatmap_md5 = ? AND s.play_mode = ?
                AND s.userid != ? AND s.completed = 3 AND u.privileges & 1
                ORDER BY s.{sort} DESC LIMIT 1
            "#
        );

        let new_first_place: Option<NewFirstPlace> = sqlx::query_as(&query)
            .bind(first_place.beatmap_md5)
            .bind(first_place.mode)
            .bind(user_id)
            .fetch_optional(ctx.db())
            .await?;
        match new_first_place {
            None => remove_first_place(ctx, first_place.scoreid).await?,
            Some(new) => transfer_first_place(ctx, first_place.scoreid, new.id, new.userid).await?,
        }
    }

    Ok(())
}

pub async fn transfer_first_place<C: Context>(
    ctx: &C,
    old_first_score_id: i64,
    new_first_score_id: i64,
    new_user_id: i64,
) -> sqlx::Result<()> {
    sqlx::query("UPDATE scores_first SET scoreid = ?, userid = ? WHERE scoreid = ?")
        .bind(new_first_score_id)
        .bind(new_user_id)
        .bind(old_first_score_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn remove_first_place<C: Context>(ctx: &C, score_id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM scores_first WHERE scoreid = ?")
        .bind(score_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn fetch_user_scores<C: Context>(
    ctx: &C,
    user_id: i64,
    custom_gamemode: CustomGamemode,
) -> sqlx::Result<Vec<MinimalScore>> {
    let table_name = SCORES_TABLES[custom_gamemode as usize];
    let query = format!(
        r#"
            SELECT s.id AS score_id, s.score, s.pp, s.play_mode AS mode,
            s.time, s.userid AS user_id, s.beatmap_md5 FROM {table_name} s
            LEFT JOIN beatmaps b USING(beatmap_md5)
            WHERE s.userid = ? AND s.completed = 3
            AND s.score > 0 AND b.ranked > 1
        "#
    );
    sqlx::query_as(&query)
        .bind(user_id)
        .fetch_all(ctx.db())
        .await
}

pub async fn fetch_first_place<C: Context>(
    ctx: &C,
    beatmap_md5: &str,
    gamemode: Gamemode,
) -> sqlx::Result<Option<MinimalScore>> {
    let mode = gamemode.to_bancho();
    let custom_gamemode = gamemode.custom_gamemode();
    let table_name = SCORES_TABLES[custom_gamemode as usize];

    let query = format!(
        r#"
            SELECT s.id AS score_id, s.score, s.pp, s.play_mode AS mode,
            s.time, s.userid AS user_id, s.beatmap_md5 FROM scores_first
            INNER JOIN {table_name} s ON s.id = scores_first.scoreid
            INNER JOIN users ON users.id = scores_first.userid
            WHERE scores_first.beatmap_md5 = ?
            AND scores_first.mode = ?
            AND scores_first.rx = ?
            AND users.privileges & 3 = 3
            LIMIT 1
        "#
    );
    sqlx::query_as(&query)
        .bind(beatmap_md5)
        .bind(mode as u8)
        .bind(custom_gamemode as u8)
        .fetch_optional(ctx.db())
        .await
}

pub async fn replace_first_place<C: Context>(
    ctx: &C,
    score_id: i64,
    user_id: i64,
    beatmap_md5: &str,
    gamemode: Gamemode,
) -> sqlx::Result<()> {
    let mode = gamemode.to_bancho();
    let custom_gamemode = gamemode.custom_gamemode();
    sqlx::query("REPLACE INTO scores_first VALUES (?, ?, ?, ?, ?)")
        .bind(beatmap_md5)
        .bind(mode as u8)
        .bind(custom_gamemode as u8)
        .bind(score_id)
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}
