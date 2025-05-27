use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{sessions, streams};
use bancho_protocol::messages::server::UserLogout;

pub async fn handle(ctx: &RequestContext, session: &Session) -> EventResult {
    sessions::delete(ctx, session).await?;
    let logout_notification = UserLogout::new(session.user_id as _);
    streams::broadcast_message(ctx, StreamName::Main, logout_notification, None, None).await?;
    Ok(None)
}
