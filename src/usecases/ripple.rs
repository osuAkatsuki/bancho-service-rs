use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult};
use crate::models::ripple::{FetchPlayerMatchDetailsArgs, PlayerMatchDetails};
use crate::usecases::{multiplayer, sessions};

pub async fn fetch_player_match_details<C: Context>(
    ctx: &C,
    args: FetchPlayerMatchDetailsArgs,
) -> ServiceResult<PlayerMatchDetails> {
    let session = sessions::fetch_primary_by_user_id(ctx, args.user_id).await?;

    let match_id = multiplayer::fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    let last_game_id = mp_match.last_game_id.ok_or(AppError::MultiplayerNotFound)?;

    let (slot_id, slot) =
        multiplayer::fetch_session_slot(ctx, mp_match.match_id, session.session_id).await?;

    Ok(PlayerMatchDetails {
        match_id,
        match_name: mp_match.name,
        game_id: last_game_id,
        slot_id: slot_id as _,
        team: slot.team as _,
    })
}
