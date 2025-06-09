use crate::common::context::Context;
use std::net::IpAddr;

pub async fn create<C: Context>(ctx: &C, user_id: i64, ip_addr: IpAddr) -> sqlx::Result<()> {
    const QUERY: &str = const_str::concat!(
        "INSERT INTO ip_user (userid, ip, occurencies) VALUES (?, ?, 1) ",
        "ON DUPLICATE KEY UPDATE occurencies = occurencies + 1",
    );
    sqlx::query(QUERY)
        .bind(user_id)
        .bind(ip_addr.to_string())
        .execute(ctx.db())
        .await?;
    Ok(())
}
