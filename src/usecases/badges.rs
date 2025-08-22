use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::models::badges::{Badge, UserBadge};
use crate::repositories::badges;

pub async fn fetch_badge_by_name<C: Context>(ctx: &C, name: &str) -> ServiceResult<Badge> {
    match badges::fetch_badge_by_name(ctx, name).await {
        Ok(badge) => Ok(Badge::from(badge)),
        Err(sqlx::Error::RowNotFound) => Err(AppError::BadgesNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_user_badges<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<Vec<UserBadge>> {
    match badges::fetch_user_badges(ctx, user_id).await {
        Ok(user_badges) => Ok(user_badges.into_iter().map(UserBadge::from).collect()),
        Err(e) => unexpected(e),
    }
}

pub async fn add_user_badge<C: Context>(
    ctx: &C,
    user_id: i64,
    badge_name: &str,
) -> ServiceResult<()> {
    // First, get the badge by name
    let badge = fetch_badge_by_name(ctx, badge_name).await?;

    // Check if user already has this badge
    let has_badge = badges::has_user_badge(ctx, user_id, badge.id).await?;
    if has_badge {
        return Ok(()); // User already has this badge
    }

    // Add the badge
    match badges::add_user_badge(ctx, user_id, badge.id).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn remove_user_badge<C: Context>(
    ctx: &C,
    user_id: i64,
    badge_name: &str,
) -> ServiceResult<()> {
    // First, get the badge by name
    let badge = fetch_badge_by_name(ctx, badge_name).await?;

    // Remove the badge
    match badges::remove_user_badge(ctx, user_id, badge.id).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}
