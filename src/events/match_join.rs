use crate::common::context::Context;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;
use bancho_protocol::concat_messages;
use bancho_protocol::messages::client::JoinMatch;
use bancho_protocol::messages::server::{ChannelJoinSuccess, MatchJoinSuccess};

pub async fn handle<C: Context>(ctx: &C, session: &Session, args: JoinMatch<'_>) -> EventResult {
    let (mp_match, slots) =
        multiplayer::join(ctx, session, args.match_id as _, args.password).await?;
    let mp_match = mp_match.as_bancho(slots);
    let response = concat_messages!(
        MatchJoinSuccess(&mp_match),
        ChannelJoinSuccess {
            name: "#multiplayer"
        },
    );
    Ok(Some(response))
}
