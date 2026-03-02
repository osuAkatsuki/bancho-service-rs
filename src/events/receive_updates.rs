use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::{PresenceFilter, Session};
use crate::usecases::sessions;
use bancho_protocol::messages::client::ReceiveUpdates;

pub async fn handle(
    ctx: &RequestContext,
    session: &mut Session,
    args: ReceiveUpdates,
) -> EventResult {
    let filter = PresenceFilter::from(args.filter);
    if filter == session.presence_filter {
        return Ok(None);
    }

    session.presence_filter = filter;
    sessions::update(ctx, session.clone()).await?;

    Ok(None)
}
