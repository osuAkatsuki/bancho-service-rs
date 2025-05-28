use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;

pub async fn handle(_ctx: &RequestContext, _session: &Session) -> EventResult {
    Ok(None)
}
