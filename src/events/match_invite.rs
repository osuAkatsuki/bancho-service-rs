use crate::common::context::Context;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{messages, multiplayer, sessions, streams};
use bancho_protocol::messages::client::MatchInvite;
use bancho_protocol::messages::server::ChatMessage;
use bancho_protocol::structures::IrcMessage;

pub async fn handle<C: Context>(ctx: &C, session: &Session, args: MatchInvite) -> EventResult {
    if !session.is_publicly_visible() {
        return Err(AppError::InteractionBlocked);
    }
    if session.is_silenced() {
        return Err(AppError::MessagesUserSilenced);
    }
    let match_id = multiplayer::fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    let target_sessions = sessions::fetch_by_user_id(ctx, args.user_id as _).await?;
    let invite = mp_match.invite_message();

    for target_session in target_sessions {
        if messages::check_private_recipient(ctx, session, &target_session)
            .await
            .is_err()
        {
            continue;
        }
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
    }
    Ok(None)
}
