use crate::common::context::Context;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{multiplayer, sessions, streams};
use bancho_protocol::messages::client::MatchInvite;
use bancho_protocol::messages::server::ChatMessage;
use bancho_protocol::structures::IrcMessage;

pub async fn handle<C: Context>(ctx: &C, session: &Session, args: MatchInvite) -> EventResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    let target_session = sessions::fetch_one_by_user_id(ctx, args.user_id as _).await?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    let safe_password = mp_match.password.replace(" ", "_");
    let invite = format!(
        "\x01ACTION has invited you to their multiplayer match: [osump://{}/{} {}]",
        mp_match.ingame_match_id(),
        safe_password,
        mp_match.name,
    );
    let invite_message = IrcMessage {
        sender: &session.username,
        sender_id: session.user_id as _,
        text: &invite,
        recipient: &target_session.username,
    };
    streams::broadcast_message(
        ctx,
        StreamName::User(target_session.session_id),
        ChatMessage(&invite_message),
        None,
        None,
    )
    .await?;
    Ok(None)
}
