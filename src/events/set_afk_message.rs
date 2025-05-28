use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use bancho_protocol::messages::client::SetAwayMessage;
use tracing::info;

pub async fn handle(
    _ctx: &RequestContext,
    session: &Session,
    args: SetAwayMessage<'_>,
) -> EventResult {
    info!(user_id = session.user_id, "AFK Message: {:?}", args);
    Ok(None)
}
