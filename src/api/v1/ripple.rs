use crate::api::RequestContext;
use crate::common::error::{AppError, ServiceResponse};
use crate::entities::bot;
use crate::entities::channels::ChannelName;
use crate::models::ripple::{
    BaseSuccessData, FetchPlayerMatchDetailsArgs, IsOnlineArgs, IsOnlineResponse, IsVerifiedArgs,
    OnlineUsersResponse, PlayerMatchDetailsResponse, SendChatbotMessageArgs,
    VerifiedStatusResponse,
};
use crate::settings::AppSettings;
use crate::usecases::{channels, ripple, sessions, streams, users};
use axum::Json;
use axum::extract::Query;
use bancho_protocol::messages::server::ChatMessage;
use bancho_protocol::structures::IrcMessage;

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

/// This is a bit weird because score-service always wants a 200
/// https://github.com/osuAkatsuki/score-service/blob/master/app/adapters/bancho_service.py#L29
pub async fn player_match_details(
    ctx: RequestContext,
    Query(args): Query<FetchPlayerMatchDetailsArgs>,
) -> ServiceResponse<PlayerMatchDetailsResponse> {
    let mut response = PlayerMatchDetailsResponse::default();
    match ripple::fetch_player_match_details(&ctx, args).await {
        Ok(details) => response.result = Some(details),
        Err(AppError::MultiplayerUserNotInMatch | AppError::MultiplayerNotFound) => {
            response.base.message = "match not found";
        }
        Err(AppError::SessionsNotFound) => {
            response.base.message = "online user (token) not found";
        }
        Err(e) => return Err(e),
    }
    Ok(Json(response))
}

pub async fn send_chatbot_message(
    ctx: RequestContext,
    Query(args): Query<SendChatbotMessageArgs>,
) -> ServiceResponse<BaseSuccessData> {
    let settings = AppSettings::get();
    if args.key != settings.app_ci_key {
        return Err(AppError::Unauthorized);
    }

    if !args.channel.starts_with('#') {
        return Err(AppError::ChannelsInvalidName);
    }

    let channel_name = ChannelName::Chat(&args.channel);
    let channel = channels::fetch_one(&ctx, channel_name).await?;
    let msg_stream = channel_name.get_message_stream();
    let msg = IrcMessage {
        sender: bot::BOT_NAME,
        sender_id: bot::BOT_ID as _,
        text: &args.content,
        recipient: &channel.name,
    };
    streams::broadcast_message(&ctx, msg_stream, ChatMessage(&msg), None, None).await?;
    Ok(Json(Default::default()))
}
