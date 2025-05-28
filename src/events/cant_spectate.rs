use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use bancho_protocol::messages::Message;
use bancho_protocol::messages::server::FailedSpectating;

pub async fn handle(_ctx: &RequestContext, _session: &Session) -> EventResult {
    Ok(Some(Message::serialize(FailedSpectating { user_id: 0 })))
}
