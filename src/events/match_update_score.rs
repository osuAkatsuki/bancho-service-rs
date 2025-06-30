use crate::common::context::Context;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{multiplayer, streams};
use bancho_protocol::messages::client::UpdateMatchScore;
use bancho_protocol::messages::server::MatchScoreUpdate;
use bancho_protocol::structures::SlotStatus;

pub async fn handle<C: Context>(
    ctx: &C,
    session: &Session,
    mut args: UpdateMatchScore,
) -> EventResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    let (slot_id, slot) = multiplayer::fetch_session_slot(ctx, match_id, session.session_id).await?;
    if slot.status != SlotStatus::Playing {
        return Ok(None);
    }
    args.score.slot_id = slot_id as _;
    streams::broadcast_message(
        ctx,
        StreamName::Multiplaying(match_id),
        MatchScoreUpdate(&args.score),
        None,
        None,
    )
    .await?;
    Ok(None)
}
