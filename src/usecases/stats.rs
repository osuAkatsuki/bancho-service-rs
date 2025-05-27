use crate::api::RequestContext;
use crate::common::error::{ServiceResult, unexpected};
use crate::models::Gamemode;
use crate::models::stats::Stats;
use crate::repositories::stats;

pub async fn fetch_one(ctx: &RequestContext, user_id: i64, mode: Gamemode) -> ServiceResult<Stats> {
    match stats::fetch_one(ctx, user_id, mode as _).await {
        Ok(stats) => Ok(Stats::from(stats)),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_global_rank(
    ctx: &RequestContext,
    user_id: i64,
    mode: Gamemode,
) -> ServiceResult<usize> {
    match stats::fetch_global_rank(ctx, user_id, mode).await {
        Ok(Some(rank)) => Ok(rank + 1),
        Ok(None) => Ok(0),
        Err(e) => unexpected(e),
    }
}
