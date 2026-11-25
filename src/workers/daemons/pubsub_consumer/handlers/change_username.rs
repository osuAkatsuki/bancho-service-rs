use crate::common::error::{AppError, ServiceResult};
use crate::common::redis_json::Json;
use crate::common::state::AppState;
use crate::repositories::streams::StreamName;
use crate::usecases::{presences, sessions, streams, users};
use bancho_protocol::structures::Action;
use redis::Msg;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
struct ChangeUsernameArgs {
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "newUsername")]
    pub new_username: String,
}

pub async fn handle(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let args: Json<ChangeUsernameArgs> = msg.get_payload()?;
    let args = args.into_inner();

    info!(
        user_id = args.user_id,
        new_username = args.new_username,
        "Handling change username event for user",
    );
    let user = users::fetch_one(&ctx, args.user_id).await?;
    match presences::fetch_one(&ctx, user.user_id).await {
        Ok(presence)
            if presence.action.action == Action::Playing
                || presence.action.action == Action::Multiplaying =>
        {
            users::queue_username_change(&ctx, user.user_id, &args.new_username).await?;
        }
        Ok(_) => {
            users::change_username(&ctx, user.user_id, &args.new_username).await?;
            sync_live_username(
                &ctx,
                user.user_id,
                &args.new_username,
                user.privileges.is_publicly_visible(),
            )
            .await?;
        }
        Err(AppError::PresencesNotFound) => {
            users::change_username(&ctx, user.user_id, &args.new_username).await?;
        }
        Err(e) => return Err(e),
    }

    info!(
        user_id = args.user_id,
        new_username = args.new_username,
        "Successfully handled change username event for user",
    );

    Ok(())
}

pub async fn handle_committed(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let user_id: i64 = msg.get_payload()?;
    let user = users::fetch_one(&ctx, user_id).await?;
    sync_live_username(
        &ctx,
        user_id,
        &user.username,
        user.privileges.is_publicly_visible(),
    )
    .await
}

async fn sync_live_username(
    ctx: &AppState,
    user_id: i64,
    username: &str,
    publicly_visible: bool,
) -> ServiceResult<()> {
    let sessions = sessions::fetch_by_user_id(ctx, user_id).await?;
    for session in sessions {
        sessions::update(ctx, session.session_id, |current| {
            current.username = username.to_owned();
        })
        .await?;
    }

    match presences::fetch_one(ctx, user_id).await {
        Ok(mut presence) => {
            presence.username = username.to_owned();
            let presence = presences::update(ctx, presence).await?;
            if publicly_visible {
                streams::broadcast_data(ctx, StreamName::Main, &presence.user_panel(), None, None)
                    .await?;
            }
        }
        Err(AppError::PresencesNotFound) => {}
        Err(e) => return Err(e),
    }
    Ok(())
}
