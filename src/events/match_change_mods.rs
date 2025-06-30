use crate::common::context::Context;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;
use bancho_protocol::messages::client::MatchChangeMods;

pub async fn handle<C: Context>(ctx: &C, session: &Session, args: MatchChangeMods) -> EventResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    multiplayer::change_mods(ctx, match_id, args.mods, Some(session.identity())).await?;
    Ok(None)
}
