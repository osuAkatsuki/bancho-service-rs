use crate::common::context::Context;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;
use bancho_protocol::messages::client::MatchLock;
use bancho_protocol::structures::SlotStatus;

pub async fn handle<C: Context>(ctx: &C, session: &Session, args: MatchLock) -> EventResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    multiplayer::set_slot_status(
        ctx,
        match_id,
        args.slot_id as _,
        SlotStatus::Locked,
        Some(session.user_id),
    )
    .await?;
    Ok(None)
}
