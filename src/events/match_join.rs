use crate::common::context::Context;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;
use bancho_protocol::messages::MessageArgs;
use bancho_protocol::messages::client::JoinMatch;
use bancho_protocol::messages::server::MatchJoinSuccess;
use bancho_protocol::serde::BinarySerialize;

pub async fn handle<C: Context>(ctx: &C, session: &Session, args: JoinMatch<'_>) -> EventResult {
    let (mp_match, slots) =
        multiplayer::join(ctx, session, args.match_id as _, args.password).await?;
    let mp_match = mp_match.to_bancho(slots);
    Ok(Some(MatchJoinSuccess(&mp_match).as_message().serialize()))
}
