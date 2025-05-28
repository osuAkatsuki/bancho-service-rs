use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::messages::Recipient;
use crate::models::sessions::Session;
use crate::usecases::{channels, messages};
use bancho_protocol::messages::client::PublicChatMessage;

pub async fn handle(
    ctx: &RequestContext,
    session: &Session,
    args: PublicChatMessage<'_>,
) -> EventResult {
    let channel_name = channels::get_channel_name(ctx, session, &args.message.recipient).await?;
    messages::send_bancho(ctx, session, Recipient::Channel(channel_name), args.message).await
}
