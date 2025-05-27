use crate::api::RequestContext;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::models::users::User;
use crate::repositories::users;

pub async fn fetch_one(ctx: &RequestContext, user_id: i64) -> ServiceResult<User> {
    match users::fetch_one(ctx, user_id).await {
        Ok(user) => User::try_from(user),
        Err(sqlx::Error::RowNotFound) => Err(AppError::UsersNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_one_by_username(ctx: &RequestContext, username: &str) -> ServiceResult<User> {
    match users::fetch_one_by_username(ctx, username).await {
        Ok(user) => User::try_from(user),
        Err(sqlx::Error::RowNotFound) => Err(AppError::UsersNotFound),
        Err(e) => unexpected(e),
    }
}
