use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::models::multiplayer::{MultiplayerMatch, MultiplayerMatchSlot, MultiplayerMatchSlots};
use crate::repositories::multiplayer;

pub async fn fetch_all<C: Context>(ctx: &C) -> ServiceResult<Vec<MultiplayerMatch>> {
    match multiplayer::fetch_all(ctx).await {
        Ok(matches) => matches.map(MultiplayerMatch::try_from).collect(),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_all_with_slots<C: Context>(
    ctx: &C,
) -> ServiceResult<Vec<(MultiplayerMatch, MultiplayerMatchSlots)>> {
    let matches = fetch_all(ctx).await?;
    let mut result = vec![];
    for mp_match in matches {
        let slots = fetch_all_slots(ctx, mp_match.match_id).await?;
        result.push((mp_match, slots));
    }
    Ok(result)
}

pub async fn fetch_all_slots<C: Context>(
    ctx: &C,
    match_id: i64,
) -> ServiceResult<MultiplayerMatchSlots> {
    match multiplayer::fetch_all_slots(ctx, match_id).await {
        Ok(slots) => Ok(MultiplayerMatchSlot::from(slots)),
        Err(e) => unexpected(e),
    }
}
