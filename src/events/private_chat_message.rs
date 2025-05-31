use crate::api::RequestContext;
use crate::common::error::AppError;
use crate::entities::bot;
use crate::events::EventResult;
use crate::models::messages::Recipient;
use crate::models::sessions::Session;
use crate::usecases::{messages, sessions};
use bancho_protocol::messages::client::PrivateChatMessage;

pub async fn handle(
    ctx: &RequestContext,
    session: &mut Session,
    args: PrivateChatMessage<'_>,
) -> EventResult {
    let recipient_name = args.message.recipient;
    if recipient_name == bot::BOT_NAME {
        messages::send_bancho(ctx, session, Recipient::Bot, args.message).await
    } else {
        let recipient_session = sessions::fetch_one_by_username(ctx, recipient_name).await;
        let recipient = match recipient_session {
            Ok(ref recipient_session) => Recipient::UserSession(recipient_session),
            Err(AppError::SessionsNotFound) => Recipient::OfflineUser(recipient_name),
            Err(e) => return Err(e),
        };
        messages::send_bancho(ctx, session, recipient, args.message).await
    }
}
