use crate::common::context::Context;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;
use bancho_protocol::messages::client::MatchChangeHost;

pub async fn handle<C: Context>(ctx: &C, session: &Session, args: MatchChangeHost) -> EventResult {
    if args.slot_id > 15 {
        return Err(AppError::MultiplayerInvalidSlotID);
    }

    let match_id = multiplayer::fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    multiplayer::transfer_host_to_slot(ctx, match_id, args.slot_id as _, Some(session.user_id))
        .await?;
    Ok(None)
}
