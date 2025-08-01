use crate::common::context::Context;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;
use bancho_protocol::messages::Message;
use bancho_protocol::messages::server::ChannelKick;

pub async fn handle<C: Context>(ctx: &C, session: &Session, _args: ()) -> EventResult {
    multiplayer::leave(ctx, session, None).await?;
    Ok(Some(Message::serialize(ChannelKick {
        name: "#multiplayer",
    })))
}
