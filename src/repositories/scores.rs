use crate::common::context::Context;
use crate::entities::gamemodes::CustomGamemode;
use crate::entities::scores::{FirstPlaceScore, NewFirstPlace};
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
