use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(sqlx::FromRow)]
pub struct HardwareLog {
    #[sqlx(rename = "userid")]
    pub user_id: i64,
    #[sqlx(rename = "mac")]
    pub adapters_md5: String,
    #[sqlx(rename = "unique_id")]
    pub uninstall_md5: String,
    #[sqlx(rename = "disk_id")]
    pub disk_signature_md5: String,
    pub occurencies: Decimal,
    pub activated: bool,
    pub last_used: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct MatchingHardwareLog {
    #[sqlx(rename = "userid")]
    pub user_id: i64,
    pub username: String,
    #[sqlx(rename = "privileges")]
    pub user_privileges: i32,
    #[sqlx(rename = "mac")]
    pub adapters_md5: String,
    #[sqlx(rename = "unique_id")]
    pub uninstall_md5: String,
    #[sqlx(rename = "disk_id")]
    pub disk_signature_md5: String,
    pub occurencies: Decimal,
    pub activated: bool,
    pub last_used: DateTime<Utc>,
}
