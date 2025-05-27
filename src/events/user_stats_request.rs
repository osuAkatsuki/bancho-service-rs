use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use bancho_protocol::messages::client::UserStatsRequest;
use tracing::info;

pub async fn handle(
    ctx: &RequestContext,
    session: &Session,
    args: UserStatsRequest,
) -> EventResult {
    info!("{:?}", args.user_ids);
    Ok(None)
}
