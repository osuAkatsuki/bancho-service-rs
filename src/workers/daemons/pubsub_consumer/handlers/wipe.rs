use crate::common::error::{AppError, ServiceResult};
use crate::common::state::AppState;
use crate::entities::gamemodes::{CustomGamemode, Gamemode};
use crate::usecases::{scores, stats, users};
use bancho_protocol::structures::Mode;
use redis::Msg;
use std::str::FromStr;
use tracing::info;

pub async fn handle(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let payload = msg.get_payload_bytes();
    let payload_str = std::str::from_utf8(payload)?;
    let mut split = payload_str.split(',');

    let user_id = split.next().ok_or(AppError::DecodingRequestFailed)?;
    let rx = split.next().ok_or(AppError::DecodingRequestFailed)?;
    let gm = split.next().ok_or(AppError::DecodingRequestFailed)?;

    let user_id = i64::from_str(user_id)?;
    let rx = u8::from_str(rx)?;
    let gm = u8::from_str(gm)?;

    let user = users::fetch_one(&ctx, user_id).await?;
    let mode = Mode::try_from(gm)?;
    let custom_mode = CustomGamemode::try_from(rx)?;
    let gamemode = Gamemode::from(mode, custom_mode);
    info!(user_id, "Handling wipe event for user");

    scores::remove_first_places(&ctx, user.user_id, Some(mode), Some(custom_mode)).await?;
    stats::remove_from_leaderboard(&ctx, user.user_id, user.country, gamemode).await?;

    info!(user_id, "Successfully handled wipe event for user");
    Ok(())
}
