use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::sessions;
use bancho_protocol::messages::client::ToggleBlockNonFriendDms;

pub async fn handle(
    ctx: &RequestContext,
    session: &Session,
    args: ToggleBlockNonFriendDms,
) -> EventResult {
    let private_dms = args.val != 0;
    sessions::set_private_dms(ctx, session, private_dms).await?;
    Ok(None)
}
