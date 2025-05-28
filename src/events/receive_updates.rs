use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use bancho_protocol::messages::client::ReceiveUpdates;

pub async fn handle(
    _ctx: &RequestContext,
    _session: &Session,
    _args: ReceiveUpdates,
) -> EventResult {
    Ok(None)
}
