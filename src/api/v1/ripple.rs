use crate::api::RequestContext;
use crate::common::error::ServiceResponse;
use crate::models::ripple::{
    IsOnlineArgs, IsOnlineResponse, IsVerifiedArgs, OnlineUsersResponse, VerifiedStatusResponse,
};
use crate::usecases::{sessions, users};
use axum::Json;
use axum::extract::Query;

pub async fn is_online(
    ctx: RequestContext,
    Query(args): Query<IsOnlineArgs>,
) -> ServiceResponse<IsOnlineResponse> {
    let is_online = sessions::is_online(&ctx, args.user_id).await?;
    Ok(Json(IsOnlineResponse {
        result: is_online,
        ..Default::default()
    }))
}

pub async fn online_users(ctx: RequestContext) -> ServiceResponse<OnlineUsersResponse> {
    let online_count = sessions::fetch_count(&ctx).await?;
    Ok(Json(OnlineUsersResponse {
        result: online_count,
        ..Default::default()
    }))
}

/// Only here for legacy reasons
pub async fn server_status(_ctx: RequestContext) -> ServiceResponse<OnlineUsersResponse> {
    Ok(Json(OnlineUsersResponse {
        result: 1,
        ..Default::default()
    }))
}

pub async fn verified_status(
    ctx: RequestContext,
    Query(args): Query<IsVerifiedArgs>,
) -> ServiceResponse<VerifiedStatusResponse> {
    let verified_status = users::fetch_verified_status(&ctx, args.user_id).await?;
    Ok(Json(VerifiedStatusResponse {
        result: verified_status as _,
        ..Default::default()
    }))
}
