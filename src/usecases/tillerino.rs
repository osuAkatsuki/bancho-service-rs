use crate::adapters::beatmaps_service;
use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::entities::tillerino::LastNowPlayingState;
use crate::models::tillerino::NowPlayingMessage;
use crate::repositories::tillerino;
use uuid::Uuid;

pub async fn save_np<C: Context>(
    ctx: &C,
    session_id: Uuid,
    np: NowPlayingMessage,
) -> ServiceResult<LastNowPlayingState> {
    let beatmap = beatmaps_service::fetch_by_id(np.beatmap_id).await?;
    let last_np = LastNowPlayingState {
        beatmap_id: np.beatmap_id,
        beatmap_set_id: beatmap.beatmapset_id,
        beatmap_md5: beatmap.beatmap_md5,
        mode: np.mode as _,
        mods: np.mods.bits(),
    };
    match tillerino::save_np(ctx, session_id, last_np).await {
        Ok(last_np) => Ok(last_np),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_last_np<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> ServiceResult<LastNowPlayingState> {
    match tillerino::fetch_last_np(ctx, session_id).await {
        Ok(last_np) => Ok(last_np),
        Err(e) => unexpected(e),
    }
}
