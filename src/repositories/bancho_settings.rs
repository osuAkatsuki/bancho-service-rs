use crate::common::context::{Context, PoolContext};
use crate::entities::bancho_settings::BanchoSetting;

pub async fn fetch<C: Context>(ctx: &C, key: &str) -> sqlx::Result<BanchoSetting> {
    const QUERY: &str =
        "SELECT id, name, value_int, value_string FROM bancho_settings WHERE name = ?";
    sqlx::query_as(QUERY).bind(key).fetch_one(ctx.db()).await
}

pub async fn update_int<C: Context>(ctx: &C, key: &str, value: i32) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE bancho_settings SET value_int = ? WHERE name = ?";
    sqlx::query(QUERY)
        .bind(value)
        .bind(key)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn update_str<C: Context>(ctx: &C, key: &str, value: &str) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE bancho_settings SET value_string = ? WHERE name = ?";
    sqlx::query(QUERY)
        .bind(value)
        .bind(key)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn update<C: Context>(
    ctx: &C,
    key: &str,
    value_int: i32,
    value_str: &str,
) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE bancho_settings SET value_int = ?, value_string = ? WHERE name = ?";
    sqlx::query(QUERY)
        .bind(value_int)
        .bind(value_str)
        .bind(key)
        .execute(ctx.db())
        .await?;
    Ok(())
}
