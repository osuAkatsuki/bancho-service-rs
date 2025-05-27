use crate::api::RequestContext;
use crate::entities::users::User;

const TABLE_NAME: &str = "users";
const READ_FIELDS: &str = r#"
id, username, username_safe, password_md5, email,
register_datetime, latest_activity, silence_end, silence_reason,
privileges, donor_expire, frozen, notes, ban_datetime,
previous_overwrite, whitelist, clan_id, userpage_content,
userpage_allowed, freeze_reason, country, username_aka,
can_custom_badge, show_custom_badge, custom_badge_icon, custom_badge_name,
favourite_mode, play_style, vanilla_pp_leaderboards, has_free_username_change"#;

pub async fn fetch_one(ctx: &RequestContext, user_id: i64) -> sqlx::Result<User> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " WHERE id = ?"
    );
    sqlx::query_as(QUERY).bind(user_id).fetch_one(&ctx.db).await
}

pub async fn fetch_one_by_username(ctx: &RequestContext, username: &str) -> sqlx::Result<User> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " WHERE username = ?"
    );
    sqlx::query_as(QUERY)
        .bind(username)
        .fetch_one(&ctx.db)
        .await
}
