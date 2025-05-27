use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub username_safe: String,
    pub email: String,
    pub password_md5: String,
    pub register_datetime: i64,
    pub latest_activity: i64,
    #[sqlx(default)]
    pub silence_end: Option<i64>,
    #[sqlx(default)]
    pub silence_reason: Option<String>,
    pub privileges: i32,
    pub donor_expire: i64,
    pub frozen: bool,
    pub notes: Option<String>,
    pub ban_datetime: i64,
    pub previous_overwrite: i64,
    pub whitelist: i8,
    pub clan_id: i64,
    pub userpage_allowed: bool,
    pub userpage_content: Option<Vec<u8>>,
    pub freeze_reason: Option<String>,
    pub country: String,
    pub can_custom_badge: bool,
    pub show_custom_badge: bool,
    pub custom_badge_icon: String,
    pub custom_badge_name: String,
    pub favourite_mode: i16,
    pub play_style: i16,
    pub vanilla_pp_leaderboards: bool,
    pub has_free_username_change: bool,
}
