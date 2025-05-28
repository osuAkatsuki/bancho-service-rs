use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::relationships;
use bancho_protocol::messages::client::AddFriend;

pub async fn handle(ctx: &RequestContext, session: &Session, args: AddFriend) -> EventResult {
    relationships::add_friend(ctx, session.user_id, args.target_id as _).await?;
    Ok(None)
}
