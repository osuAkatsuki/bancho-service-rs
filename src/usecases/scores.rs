use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::entities::gamemodes::CustomGamemode;
use crate::repositories::scores;
use bancho_protocol::structures::Mode;

pub async fn remove_first_places<C: Context>(
    ctx: &C,
    user_id: i64,
    mode: Option<Mode>,
    custom_gamemode: Option<CustomGamemode>,
) -> ServiceResult<()> {
    match scores::remove_first_places(ctx, user_id, mode, custom_gamemode).await {
        Ok(()) => Ok(()),
        Err(e) => unexpected(e),
    }
}
