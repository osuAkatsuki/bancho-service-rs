use crate::common::context::{Context, PoolContext};
use crate::entities::hardware_logs::{HardwareLog, MatchingHardwareLog};

/// Fetches hardware log entries not matching the user_id but matching either of the hashes
pub async fn fetch_foreign_matching_hardware<C: Context>(
    ctx: &C,
    user_id: i64,
    mac: &str,
    unique_id: &str,
    disk_id: &str,
) -> sqlx::Result<Vec<MatchingHardwareLog>> {
    // md5("runningunderwine"), osu! is running on wine
    if mac == "b4ec3c4334a0249dae95c284ec5983df" {
        // Only match by unique_id
        // TODO: is matching by disk_id possible here?
        const QUERY: &str = const_str::concat!(
            "SELECT hw.userid, u.username, u.privileges, ",
            "hw.mac, hw.unique_id, hw.disk_id, ",
            "SUM(hw.occurencies) AS occurencies, ",
            "MAX(hw.activated) AS activated, ",
            "MAX(hw.created_at) AS last_used ",
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
            "MAX(hw.created_at) AS last_used ",
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
        "MAX(created_at) AS last_used ",
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
