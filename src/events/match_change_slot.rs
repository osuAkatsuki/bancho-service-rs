use crate::common::context::Context;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;
use bancho_protocol::messages::client::MatchChangeSlot;

pub async fn handle<C: Context>(ctx: &C, session: &Session, args: MatchChangeSlot) -> EventResult {
    if args.slot_id > 15 {
        return Err(AppError::MultiplayerSlotNotFound);
    }

    let match_id = multiplayer::fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    multiplayer::swap_user_slots(ctx, match_id, args.slot_id as _, session.user_id).await?;
    Ok(None)
}
