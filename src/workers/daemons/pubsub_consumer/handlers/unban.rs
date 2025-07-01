use crate::common::error::ServiceResult;
use crate::common::state::AppState;
use crate::entities::bot;
use crate::repositories::streams::StreamName;
use crate::usecases::{scores, sessions, streams, users};
use bancho_protocol::messages::server::ChatMessage;
use bancho_protocol::structures::IrcMessage;
use redis::Msg;
use tracing::info;

pub async fn handle(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let user_id: i64 = msg.get_payload()?;
    info!(user_id, "Handling unban event for user");

    let user = users::fetch_one(&ctx, user_id).await?;
    if user.privileges.is_publicly_visible() {
        scores::recalculate_user_first_places(&ctx, user.user_id).await?;

        let sessions = sessions::fetch_by_user_id(&ctx, user.user_id).await?;
        let unrestriction_notification = unrestriction_message(&user.username);
        for mut session in sessions {
            session.privileges = user.privileges;
            let session = sessions::update(&ctx, session).await?;

            streams::broadcast_message(
                &ctx,
                StreamName::User(session.session_id),
                ChatMessage(&unrestriction_notification),
                None,
                None,
            )
            .await?;
        }
    }

    info!(user_id, "Successfully handled unban event for user");
    Ok(())
}

pub const fn unrestriction_message(recipient: &str) -> IrcMessage<'_> {
    IrcMessage {
        recipient,
        sender: bot::BOT_NAME,
        sender_id: bot::BOT_ID as _,
        text: "Your account is now unrestricted.",
    }
}
