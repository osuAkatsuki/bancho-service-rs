use crate::common::context::Context;
use std::net::IpAddr;

pub async fn create<C: Context>(ctx: &C, user_id: i64, ip_addr: IpAddr) -> sqlx::Result<()> {
    const QUERY: &str = "INSERT INTO ip_user (userid, ip) VALUES (?, ?)";
    sqlx::query(QUERY)
        .bind(user_id)
        .bind(ip_addr.to_string())
        .execute(ctx.db())
        .await?;
    Ok(())
}
