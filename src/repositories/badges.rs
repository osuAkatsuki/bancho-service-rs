use crate::common::context::Context;
use crate::entities::badges::{Badge, UserBadge};

pub async fn fetch_badge_by_name<C: Context>(ctx: &C, name: &str) -> sqlx::Result<Badge> {
    const QUERY: &str = "SELECT id, name, icon, colour FROM badges WHERE name = ?";
    sqlx::query_as(QUERY).bind(name).fetch_one(ctx.db()).await
}

pub async fn fetch_user_badges<C: Context>(ctx: &C, user_id: i64) -> sqlx::Result<Vec<UserBadge>> {
    const QUERY: &str = "SELECT id, user, badge FROM user_badges WHERE user = ?";
    sqlx::query_as(QUERY)
        .bind(user_id)
        .fetch_all(ctx.db())
        .await
}

pub async fn add_user_badge<C: Context>(ctx: &C, user_id: i64, badge_id: i32) -> sqlx::Result<()> {
    const QUERY: &str = "INSERT INTO user_badges (user, badge) VALUES (?, ?)";
    sqlx::query(QUERY)
        .bind(user_id)
        .bind(badge_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn remove_user_badge<C: Context>(
    ctx: &C,
    user_id: i64,
    badge_id: i32,
) -> sqlx::Result<()> {
    const QUERY: &str = "DELETE FROM user_badges WHERE user = ? AND badge = ?";
    sqlx::query(QUERY)
        .bind(user_id)
        .bind(badge_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn has_user_badge<C: Context>(
    ctx: &C,
    user_id: i64,
    badge_id: i32,
) -> sqlx::Result<bool> {
    const QUERY: &str = "SELECT COUNT(*) FROM user_badges WHERE user = ? AND badge = ?";
    let count: i64 = sqlx::query_scalar(QUERY)
        .bind(user_id)
        .bind(badge_id)
        .fetch_one(ctx.db())
        .await?;
    Ok(count > 0)
}
