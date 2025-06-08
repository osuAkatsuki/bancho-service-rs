use crate::common::context::Context;
use crate::entities::hardware_logs::{HardwareLog, MultiaccountQueryEntity};

pub async fn fetch_potential_multiaccounts<C: Context>(
    ctx: &C,
    user_id: i64,
    mac: &str,
    unique_id: &str,
    disk_id: &str,
) -> sqlx::Result<Vec<MultiaccountQueryEntity>> {
    const QUERY: &str = const_str::concat!(
        "SELECT hw.userid, u.username, u.privileges, ",
        "hw.mac, hw.unique_id, hw.disk_id, ",
        "SUM(hw.occurencies) AS occurencies, ",
        "MAX(hw.activated) AS activated, ",
        "MAX(hw.created_at) AS last_used ",
        "FROM hw_user hw ",
        "INNER JOIN users u ON hw.userid = u.id ",
        "WHERE hw.userid != ? AND (hw.mac = ? OR hw.unique_id = ? OR hw.disk_id = ?) ",
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

pub async fn fetch_one<C: Context>(
    ctx: &C,
    user_id: i64,
    mac: &str,
    unique_id: &str,
    disk_id: &str,
) -> sqlx::Result<HardwareLog> {
    const QUERY: &str = const_str::concat!(
        "SELECT userid, mac, unique_id, disk_id, ",
        "SUM(occurencies) AS occurencies, ",
        "MAX(activated) AS activated, ",
        "MAX(created_at) AS last_used ",
        "FROM hw_user ",
        "WHERE userid = ? AND mac = ? AND unique_id = ? AND disk_id = ? ",
        "GROUP BY mac, unique_id, disk_id, userid"
    );
    sqlx::query_as(QUERY)
        .bind(user_id)
        .bind(mac)
        .bind(unique_id)
        .bind(disk_id)
        .fetch_one(ctx.db())
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
