use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::spectators;

pub async fn handle(ctx: &RequestContext, session: &Session, _args: ()) -> EventResult {
    spectators::leave(ctx, session, None).await?;
    Ok(None)
}
