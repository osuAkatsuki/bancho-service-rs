use crate::common::chat::safe_username;
use crate::common::context::Context;
use crate::entities::users::User;
use crate::models::privileges::Privileges;
use chrono::{TimeDelta, Utc};
use redis::AsyncCommands;

const TABLE_NAME: &str = "users";
const READ_FIELDS: &str = r#"
id, username, username_safe, password_md5, email,
register_datetime, latest_activity, silence_end, silence_reason,
privileges, donor_expire, frozen, notes, ban_datetime,
previous_overwrite, whitelist, clan_id, userpage_content,
userpage_allowed, freeze_reason, country, username_aka,
can_custom_badge, show_custom_badge, custom_badge_icon, custom_badge_name,
favourite_mode, play_style, vanilla_pp_leaderboards, has_free_username_change"#;

pub async fn fetch_one<C: Context>(ctx: &C, user_id: i64) -> sqlx::Result<User> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " WHERE id = ?"
    );
    sqlx::query_as(QUERY)
        .bind(user_id)
        .fetch_one(ctx.db())
        .await
}

pub async fn fetch_one_by_username<C: Context>(ctx: &C, username: &str) -> sqlx::Result<User> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " WHERE username = ?"
    );
    sqlx::query_as(QUERY)
        .bind(username)
        .fetch_one(ctx.db())
        .await
}

pub async fn fetch_one_by_username_safe<C: Context>(ctx: &C, username: &str) -> sqlx::Result<User> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " WHERE username_safe = ?"
    );
    sqlx::query_as(QUERY)
        .bind(username)
        .fetch_one(ctx.db())
        .await
}

pub async fn silence_user<C: Context>(
    ctx: &C,
    user_id: i64,
    silence_reason: &str,
    silence_seconds: i64,
) -> sqlx::Result<()> {
    const QUERY: &str = const_str::concat!(
        "UPDATE ",
        TABLE_NAME,
        " SET silence_reason = ?, silence_end = ? WHERE id = ?"
    );
    let silence_end = Utc::now() + TimeDelta::seconds(silence_seconds);
    let silence_end = silence_end.timestamp();
    sqlx::query(QUERY)
        .bind(silence_reason)
        .bind(silence_end)
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn verify_user<C: Context>(ctx: &C, user_id: i64) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET privileges = ? WHERE id = ?";
    let privileges = Privileges::PubliclyVisible | Privileges::CanLogin;
    sqlx::query(QUERY)
        .bind(privileges.bits())
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn restrict<C: Context>(ctx: &C, user_id: i64) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET privileges = (privileges & ~(?)) WHERE id = ?";
    let privileges = Privileges::PubliclyVisible;
    sqlx::query(QUERY)
        .bind(privileges.bits())
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn ban<C: Context>(ctx: &C, user_id: i64) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET privileges = (privileges & ~(?)) WHERE id = ?";
    let privileges = Privileges::PubliclyVisible | Privileges::CanLogin;
    sqlx::query(QUERY)
        .bind(privileges.bits())
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn unban<C: Context>(ctx: &C, user_id: i64) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET privileges = (privileges | (?)) WHERE id = ?";
    let privileges = Privileges::PubliclyVisible | Privileges::CanLogin;
    sqlx::query(QUERY)
        .bind(privileges.bits())
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn unrestrict<C: Context>(ctx: &C, user_id: i64) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET privileges = (privileges | (?)) WHERE id = ?";
    let privileges = Privileges::PubliclyVisible;
    sqlx::query(QUERY)
        .bind(privileges.bits())
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn freeze<C: Context>(ctx: &C, user_id: i64, reason: &str) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET frozen = 1, freeze_reason = ? WHERE id = ?";
    sqlx::query(QUERY)
        .bind(reason)
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn unfreeze<C: Context>(ctx: &C, user_id: i64) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET frozen = 0, freeze_reason = NULL WHERE id = ?";
    sqlx::query(QUERY).bind(user_id).execute(ctx.db()).await?;
    Ok(())
}

pub async fn update_privileges<C: Context>(
    ctx: &C,
    user_id: i64,
    privileges: Privileges,
) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET privileges = ? WHERE id = ?";
    sqlx::query(QUERY)
        .bind(privileges.bits())
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn update_whitelist<C: Context>(
    ctx: &C,
    user_id: i64,
    whitelist_bit: i32,
) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET whitelist = ? WHERE id = ?";
    sqlx::query(QUERY)
        .bind(whitelist_bit)
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn update_donor_expiry<C: Context>(
    ctx: &C,
    user_id: i64,
    donor_expire: i64,
) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET donor_expire = ? WHERE id = ?";
    sqlx::query(QUERY)
        .bind(donor_expire)
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn change_username<C: Context>(
    ctx: &C,
    user_id: i64,
    new_username: &str,
) -> sqlx::Result<()> {
    const QUERY: &str = "UPDATE users SET username = ?, username_safe = ? WHERE id = ?";
    let safe_username = safe_username(new_username);
    sqlx::query(QUERY)
        .bind(new_username)
        .bind(safe_username)
        .bind(user_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

fn make_queued_username_change_key(user_id: i64) -> String {
    format!("ripple:change_username_pending:{user_id}")
}

pub async fn queue_username_change<C: Context>(
    ctx: &C,
    user_id: i64,
    new_username: &str,
) -> anyhow::Result<()> {
    let mut redis = ctx.redis().await?;
    let queued_username_change_key = make_queued_username_change_key(user_id);
    let _: () = redis.set(queued_username_change_key, new_username).await?;
    Ok(())
}
