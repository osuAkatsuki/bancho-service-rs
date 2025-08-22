use crate::common::context::{Context, PoolContext};
use crate::entities::user_reports::UserReport;
use chrono::Utc;

pub async fn create<C: Context>(
    ctx: &C,
    from_uid: i64,
    to_uid: i64,
    reason: String,
) -> sqlx::Result<UserReport> {
    const QUERY: &str = const_str::concat!(
        "INSERT INTO reports ",
        "(reason, time, from_uid, to_uid) ",
        "VALUES (?, ?, ?, ?)",
    );
    let now = Utc::now();
    let time = now.timestamp().to_string();
    let res = sqlx::query(QUERY)
        .bind(&reason)
        .bind(&time)
        .bind(from_uid)
        .bind(to_uid)
        .execute(ctx.db())
        .await?;
    Ok(UserReport {
        id: res.last_insert_id() as _,
        time,
        reason,
        from_uid,
        to_uid,
    })
}
