use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::models::user_reports::UserReport;
use crate::repositories::user_reports;

pub async fn create<C: Context>(
    ctx: &C,
    from_user: i64,
    to_user: i64,
    reason: String,
) -> ServiceResult<UserReport> {
    match user_reports::create(ctx, from_user, to_user, reason).await {
        Ok(report) => UserReport::try_from(report),
        Err(e) => unexpected(e),
    }
}
