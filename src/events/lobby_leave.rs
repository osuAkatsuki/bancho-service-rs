use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::streams;

pub async fn handle(ctx: &RequestContext, session: &Session) -> EventResult {
    streams::leave(ctx, session.session_id, StreamName::Lobby).await?;
    Ok(None)
}
