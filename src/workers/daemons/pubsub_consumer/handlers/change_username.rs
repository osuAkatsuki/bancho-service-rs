use crate::common::error::{AppError, ServiceResult};
use crate::common::redis_json::Json;
use crate::common::state::AppState;
use crate::repositories::streams::StreamName;
use crate::usecases::{presences, sessions, streams, users};
use bancho_protocol::messages::server::UsernameChanged;
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
    let old_username = &user.username;

    match presences::fetch_one(&ctx, user.user_id).await {
        Ok(presence)
            if presence.action.action == Action::Playing
                || presence.action.action == Action::Multiplaying =>
        {
            users::queue_username_change(&ctx, user.user_id, &args.new_username).await?;
        }
        Ok(mut presence) => {
            users::change_username(&ctx, user.user_id, &args.new_username).await?;
            presence.username = args.new_username.clone();
            presences::update(&ctx, presence).await?;
            let sessions = sessions::fetch_by_user_id(&ctx, user.user_id).await?;
            for mut session in sessions {
                session.username = args.new_username.clone();
                sessions::update(&ctx, session).await?;
            }

            if user.privileges.is_publicly_visible() {
                let username_change_notification =
                    UsernameChanged::new(old_username, &args.new_username);
                streams::broadcast_message(
                    &ctx,
                    StreamName::Main,
                    username_change_notification,
                    None,
                    None,
                )
                .await?;
            }
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
