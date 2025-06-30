use crate::api::RequestContext;
use crate::common::error::ServiceResponse;
use crate::models::ripple::{IsOnlineArgs, IsOnlineResponse, OnlineUsersResponse};
use crate::usecases::sessions;
use axum::Json;
use axum::extract::Query;

pub async fn is_online(
    ctx: RequestContext,
    Query(args): Query<IsOnlineArgs>,
) -> ServiceResponse<IsOnlineResponse> {
    let is_online = sessions::is_online(&ctx, args.id).await?;
    Ok(Json(IsOnlineResponse {
        message: "ok",
        status: 200,
        result: is_online,
    }))
}

pub async fn online_users(ctx: RequestContext) -> ServiceResponse<OnlineUsersResponse> {
    Ok(Json(OnlineUsersResponse {}))
}
