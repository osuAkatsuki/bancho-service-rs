use crate::common::context::{Context, PoolContext};
use crate::entities::hardware_logs::{HardwareLog, HardwareUser, MatchingHardwareLog, MultiUserHardware};

/// Fetches hardware log entries not matching the user_id but matching either of the hashes
pub async fn fetch_foreign_matching_hardware<C: Context>(
    ctx: &C,
    user_id: i64,
    mac: &str,
    unique_id: &str,
    disk_id: &str,
) -> sqlx::Result<Vec<MatchingHardwareLog>> {
    // unique_id = md5(md5("00000000-0000-0000-0000-000000000000"))
    // disk_id = md5(md5("0"))
    if unique_id == "06a9e146cb8cc0853ded03bb15f2260e"
        || disk_id == "dcfcd07e645d245babe887e5e2daa016"
    {
        const QUERY: &str = const_str::concat!(
            "SELECT hw.userid, u.username, u.privileges, ",
            "hw.mac, hw.unique_id, hw.disk_id, ",
            "SUM(hw.occurencies) AS occurencies, ",
            "MAX(hw.activated) AS activated, ",
            "MAX(hw.created_at) AS last_used, ",
            "MAX(hw.is_shared_device) AS is_shared_device, ",
            "MAX(hw.approved_by_admin_id) AS approved_by_admin_id, ",
            "MAX(hw.approved_at) AS approved_at, ",
            "MAX(hw.approval_reason) AS approval_reason ",
            "FROM hw_user hw ",
            "INNER JOIN users u ON hw.userid = u.id ",
            "WHERE hw.userid != ? AND hw.mac = ? AND hw.unique_id = ? AND hw.disk_id = ? ",
            "GROUP BY hw.mac, hw.unique_id, hw.disk_id, hw.userid ",
            "ORDER BY hw.userid"
        );
        sqlx::query_as(QUERY)
            .bind(user_id)
            .bind(mac)
            .bind(unique_id)
            .bind(disk_id)
            .fetch_all(ctx.db())
            .await
    } else if mac == "b4ec3c4334a0249dae95c284ec5983df" {
        // md5("runningunderwine"), osu! is running on wine
        // Only match by unique_id
        // TODO: is matching by disk_id possible here?
        const QUERY: &str = const_str::concat!(
            "SELECT hw.userid, u.username, u.privileges, ",
            "hw.mac, hw.unique_id, hw.disk_id, ",
            "SUM(hw.occurencies) AS occurencies, ",
            "MAX(hw.activated) AS activated, ",
            "MAX(hw.created_at) AS last_used, ",
            "MAX(hw.is_shared_device) AS is_shared_device, ",
            "MAX(hw.approved_by_admin_id) AS approved_by_admin_id, ",
            "MAX(hw.approved_at) AS approved_at, ",
            "MAX(hw.approval_reason) AS approval_reason ",
            "FROM hw_user hw ",
            "INNER JOIN users u ON hw.userid = u.id ",
            "WHERE hw.userid != ? AND hw.unique_id = ? ",
            "GROUP BY hw.mac, hw.unique_id, hw.disk_id, hw.userid ",
            "ORDER BY hw.userid"
        );
        sqlx::query_as(QUERY)
            .bind(user_id)
            .bind(unique_id)
            .fetch_all(ctx.db())
            .await
    } else {
        const QUERY: &str = const_str::concat!(
            "SELECT hw.userid, u.username, u.privileges, ",
            "hw.mac, hw.unique_id, hw.disk_id, ",
            "SUM(hw.occurencies) AS occurencies, ",
            "MAX(hw.activated) AS activated, ",
            "MAX(hw.created_at) AS last_used, ",
            "MAX(hw.is_shared_device) AS is_shared_device, ",
            "MAX(hw.approved_by_admin_id) AS approved_by_admin_id, ",
            "MAX(hw.approved_at) AS approved_at, ",
            "MAX(hw.approval_reason) AS approval_reason ",
            "FROM hw_user hw ",
            "INNER JOIN users u ON hw.userid = u.id ",
            "WHERE hw.userid != ? AND (hw.mac = ? OR hw.unique_id = ?) AND hw.disk_id = ? ",
            "GROUP BY hw.mac, hw.unique_id, hw.disk_id, hw.userid ",
            "ORDER BY hw.userid"
        );
        sqlx::query_as(QUERY)
            .bind(user_id)
            .bind(mac)
            .bind(unique_id)
            .bind(disk_id)
            .fetch_all(ctx.db())
            .await
    }
}

pub async fn fetch_own_matching_hardware<C: Context>(
    ctx: &C,
    user_id: i64,
    mac: &str,
    unique_id: &str,
    disk_id: &str,
) -> sqlx::Result<Vec<HardwareLog>> {
    const QUERY: &str = const_str::concat!(
        "SELECT userid, mac, unique_id, disk_id, ",
        "SUM(occurencies) AS occurencies, ",
        "MAX(activated) AS activated, ",
        "MAX(created_at) AS last_used, ",
        "MAX(is_shared_device) AS is_shared_device, ",
        "MAX(approved_by_admin_id) AS approved_by_admin_id, ",
        "MAX(approved_at) AS approved_at, ",
        "MAX(approval_reason) AS approval_reason ",
        "FROM hw_user ",
        "WHERE userid = ? AND (mac = ? OR unique_id = ? OR disk_id = ?) ",
        "GROUP BY mac, unique_id, disk_id, userid"
    );
    sqlx::query_as(QUERY)
        .bind(user_id)
        .bind(mac)
        .bind(unique_id)
        .bind(disk_id)
        .fetch_all(ctx.db())
        .await
}

pub async fn create<C: Context>(
    ctx: &C,
    user_id: i64,
    activation: bool,
    mac: &str,
    unique_id: &str,
    disk_id: &str,
) -> sqlx::Result<()> {
    const QUERY: &str = const_str::concat!(
        "INSERT INTO hw_user (userid, mac, unique_id, disk_id, activated) ",
        "VALUES (?, ?, ?, ?, ?)"
    );
    sqlx::query(QUERY)
        .bind(user_id)
        .bind(mac)
        .bind(unique_id)
        .bind(disk_id)
        .bind(activation)
        .execute(ctx.db())
        .await?;
    Ok(())
}

/// Check if a hardware combination is marked as a shared device
pub async fn is_shared_device<C: Context>(
    ctx: &C,
    mac: &str,
    unique_id: &str,
    disk_id: &str,
) -> sqlx::Result<bool> {
    const QUERY: &str = const_str::concat!(
        "SELECT EXISTS(",
        "  SELECT 1 FROM hw_user ",
        "  WHERE mac = ? AND unique_id = ? AND disk_id = ? ",
        "  AND is_shared_device = 1",
        ") AS is_shared"
    );
    let result: (bool,) = sqlx::query_as(QUERY)
        .bind(mac)
        .bind(unique_id)
        .bind(disk_id)
        .fetch_one(ctx.db())
        .await?;
    Ok(result.0)
}

/// Update shared device approval status for all hw_user entries matching this hardware
pub async fn update_shared_device_approval<C: Context>(
    ctx: &C,
    mac: &str,
    unique_id: &str,
    disk_id: &str,
    is_shared: bool,
    admin_id: Option<i64>,
    reason: Option<&str>,
) -> sqlx::Result<u64> {
    const QUERY: &str = const_str::concat!(
        "UPDATE hw_user SET ",
        "  is_shared_device = ?, ",
        "  approved_by_admin_id = ?, ",
        "  approved_at = IF(? = 1, NOW(), NULL), ",
        "  approval_reason = ? ",
        "WHERE mac = ? AND unique_id = ? AND disk_id = ?"
    );
    let result = sqlx::query(QUERY)
        .bind(is_shared)
        .bind(admin_id)
        .bind(is_shared)
        .bind(reason)
        .bind(mac)
        .bind(unique_id)
        .bind(disk_id)
        .execute(ctx.db())
        .await?;
    Ok(result.rows_affected())
}

/// Fetch all hardware entries that have multiple users (potential shared devices)
pub async fn fetch_multi_user_hardware<C: Context>(
    ctx: &C,
) -> sqlx::Result<Vec<MultiUserHardware>> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        "  hw.mac, hw.unique_id, hw.disk_id, ",
        "  COUNT(DISTINCT hw.userid) AS user_count, ",
        "  MAX(hw.is_shared_device) AS is_shared_device, ",
        "  MAX(hw.approved_by_admin_id) AS approved_by_admin_id, ",
        "  MAX(hw.approved_at) AS approved_at, ",
        "  MAX(hw.approval_reason) AS approval_reason ",
        "FROM hw_user hw ",
        "GROUP BY hw.mac, hw.unique_id, hw.disk_id ",
        "HAVING COUNT(DISTINCT hw.userid) > 1 ",
        "ORDER BY user_count DESC, hw.is_shared_device ASC"
    );
    sqlx::query_as(QUERY).fetch_all(ctx.db()).await
}

/// Fetch users for a specific hardware combination
pub async fn fetch_users_for_hardware<C: Context>(
    ctx: &C,
    mac: &str,
    unique_id: &str,
    disk_id: &str,
) -> sqlx::Result<Vec<HardwareUser>> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        "  u.id AS user_id, ",
        "  u.username, ",
        "  u.privileges, ",
        "  SUM(hw.occurencies) AS total_occurrences, ",
        "  MAX(hw.activated) AS has_activated, ",
        "  MAX(hw.created_at) AS last_used ",
        "FROM hw_user hw ",
        "INNER JOIN users u ON hw.userid = u.id ",
        "WHERE hw.mac = ? AND hw.unique_id = ? AND hw.disk_id = ? ",
        "GROUP BY u.id, u.username, u.privileges ",
        "ORDER BY total_occurrences DESC"
    );
    sqlx::query_as(QUERY)
        .bind(mac)
        .bind(unique_id)
        .bind(disk_id)
        .fetch_all(ctx.db())
        .await
}
