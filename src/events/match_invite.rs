use crate::common::context::Context;
use crate::events::EventResult;
use crate::models::sessions::Session;
use bancho_protocol::messages::client::MatchInvite;

pub async fn handle<C: Context>(ctx: &C, session: &Session, args: MatchInvite) -> EventResult {
    Ok(None)
}
