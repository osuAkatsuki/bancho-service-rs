use crate::common::error::ServiceResult;
use crate::common::state::AppState;
use crate::entities::bot;
use crate::repositories::streams::StreamName;
use crate::usecases::{scores, sessions, stats, streams, users};
use bancho_protocol::messages::server::ChatMessage;
use bancho_protocol::structures::IrcMessage;
use redis::Msg;
use tracing::info;

pub const fn restriction_message(recipient: &str) -> IrcMessage<'_> {
    IrcMessage {
        recipient,
        sender: bot::BOT_NAME,
        sender_id: bot::BOT_ID as _,
        text: "Your account is now in restricted mode. Visit the website for more information.",
    }
}

pub async fn handle(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let user_id: i64 = msg.get_payload()?;
    info!(user_id, "Handling ban event for user");

    let user = users::fetch_one(&ctx, user_id).await?;
    stats::remove_from_leaderboard(&ctx, user.user_id, user.country, None, None).await?;
    scores::remove_first_places(&ctx, user.user_id, None, None).await?;

    let sessions = sessions::fetch_by_user_id(&ctx, user_id).await?;
    let restriction_message = restriction_message(&user.username);
    for mut session in sessions {
        if !user.privileges.can_login() {
            sessions::delete(&ctx, &session).await?;
            continue;
        }

        session.privileges = user.privileges;
        let session = sessions::update(&ctx, session).await?;

        streams::broadcast_message(
            &ctx,
            StreamName::User(session.session_id),
            ChatMessage(&restriction_message),
            None,
            None,
        )
        .await?;
    }

    info!(user_id, "Successfully handled ban event for user");

    Ok(())
}
