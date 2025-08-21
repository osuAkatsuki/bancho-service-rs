use redis::AsyncCommands;

use crate::common::context::Context;
use crate::entities::beatmaps::Beatmap;

const READ_FIELDS: &str = r#"beatmap_id, beatmapset_id, beatmap_md5,
song_name, file_name, ar, od, mode, max_combo,
hit_length, bpm, ranked, latest_update, ranked_status_freezed,
playcount, passcount, rankedby, rating, bancho_ranked_status"#;

pub async fn fetch_by_id<C: Context>(ctx: &C, map_id: i32) -> sqlx::Result<Beatmap> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM beatmaps WHERE beatmap_id = ?",
    );
    let row = sqlx::query_as(QUERY)
        .bind(map_id)
        .fetch_one(ctx.db())
        .await?;
    Ok(row)
}

pub async fn fetch_by_set_id<C: Context>(ctx: &C, set_id: i32) -> sqlx::Result<Vec<Beatmap>> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM beatmaps WHERE beatmapset_id = ?",
    );
    let rows = sqlx::query_as(QUERY)
        .bind(set_id)
        .fetch_all(ctx.db())
        .await?;
    Ok(rows)
}

pub async fn update_map_ranked_status<C: Context>(
    ctx: &C,
    map_id: i32,
    new_status: i8,
) -> sqlx::Result<()> {
    const QUERY: &str =
        "UPDATE beatmaps SET ranked = ?, ranked_status_freezed = 1 WHERE beatmap_id = ?";
    sqlx::query(QUERY)
        .bind(new_status)
        .bind(map_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn update_set_ranked_status<C: Context>(
    ctx: &C,
    set_id: i32,
    new_status: i8,
) -> sqlx::Result<()> {
    const QUERY: &str =
        "UPDATE beatmaps SET ranked = ?, ranked_status_freezed = 1 WHERE beatmapset_id = ?";
    sqlx::query(QUERY)
        .bind(new_status)
        .bind(set_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn publish_map_update<C: Context>(
    ctx: &C,
    map_md5: &str,
    new_status: i8,
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let msg = format!("{map_md5},{new_status}");
    let _: () = redis.publish("cache:map_update", msg).await?;
    Ok(())
}
