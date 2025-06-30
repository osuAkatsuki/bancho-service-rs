use crate::common::context::Context;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;

pub async fn handle<C: Context>(ctx: &C, session: &Session) -> EventResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    multiplayer::switch_teams(ctx, match_id, session.session_id).await?;
    Ok(None)
}
