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
    pub is_shared_device: bool,
    pub approved_by_admin_id: Option<i64>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approval_reason: Option<String>,
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
    pub is_shared_device: bool,
    pub approved_by_admin_id: Option<i64>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approval_reason: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct MultiUserHardware {
    #[sqlx(rename = "mac")]
    pub adapters_md5: String,
    #[sqlx(rename = "unique_id")]
    pub uninstall_md5: String,
    #[sqlx(rename = "disk_id")]
    pub disk_signature_md5: String,
    pub user_count: i64,
    pub is_shared_device: bool,
    pub approved_by_admin_id: Option<i64>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approval_reason: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct HardwareUser {
    pub user_id: i64,
    pub username: String,
    pub privileges: i32,
    pub total_occurrences: Decimal,
    pub has_activated: bool,
    pub last_used: DateTime<Utc>,
}
