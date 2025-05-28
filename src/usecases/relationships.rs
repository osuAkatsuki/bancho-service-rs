use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::models::relationships::Relationship;
use crate::repositories::relationships;
use crate::usecases::users;
use tracing::warn;

pub async fn fetch_friends<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<Vec<Relationship>> {
    match relationships::fetch_friends(ctx, user_id).await {
        Ok(friends) => Ok(friends.into_iter().map(Relationship::from).collect()),
        Err(e) => unexpected(e),
    }
}

pub async fn add_friend<C: Context>(ctx: &C, user_id: i64, to_add: i64) -> ServiceResult<()> {
    let user = users::fetch_one(ctx, to_add).await?;
    if !user.privileges.is_publicly_visible() {
        warn!(
            user_id,
            to_add, "User tried to add a restricted user as a friend"
        );
        return Ok(());
    }
    match relationships::add_friend(ctx, user_id, to_add).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn remove_friend<C: Context>(ctx: &C, user_id: i64, to_remove: i64) -> ServiceResult<()> {
    match relationships::remove_friend(ctx, user_id, to_remove).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}
