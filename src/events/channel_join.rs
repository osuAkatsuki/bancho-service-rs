use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::channels;
use bancho_protocol::messages::Message;
use bancho_protocol::messages::client::JoinChannel;
use bancho_protocol::messages::server::ChannelJoinSuccess;

pub async fn handle(ctx: &RequestContext, session: &Session, args: JoinChannel<'_>) -> EventResult {
    match args.name {
        "#highlight" | "#userlog" => Ok(None),
        channel_name => {
            let name = channels::get_channel_name(ctx, session, channel_name).await?;
            channels::join(ctx, session, name).await?;
            Ok(Some(Message::serialize(ChannelJoinSuccess {
                name: channel_name,
            })))
        }
    }
}
