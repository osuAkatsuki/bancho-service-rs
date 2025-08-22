use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::sessions;
use tracing::info;

pub async fn handle(ctx: &RequestContext, session: &Session, _args: ()) -> EventResult {
    sessions::delete(ctx, session).await?;
    info!(user_id = session.user_id, "User logged out.");
    Ok(None)
}
