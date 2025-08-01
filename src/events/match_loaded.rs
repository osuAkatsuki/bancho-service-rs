use crate::common::context::Context;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;

pub async fn handle<C: Context>(ctx: &C, session: &Session, _args: ()) -> EventResult {
    multiplayer::player_loaded(ctx, session).await?;
    Ok(None)
}
