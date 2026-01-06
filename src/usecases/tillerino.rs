use crate::adapters::beatmaps_service;
use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::entities::tillerino::NowPlayingState;
use crate::models::tillerino::NowPlayingMessage;
use crate::repositories::tillerino;
use uuid::Uuid;

pub async fn save_np<C: Context>(
    ctx: &C,
    session_id: Uuid,
    np: NowPlayingMessage<'_>,
) -> ServiceResult<NowPlayingState> {
    let beatmap = beatmaps_service::fetch_by_id(np.beatmap_id).await?;
    let mode = if beatmap.mode != 0 {
        beatmap.mode
    } else {
        np.mode as _
    };
    let last_np = NowPlayingState {
        beatmap_id: np.beatmap_id,
        beatmap_set_id: beatmap.beatmapset_id,
        beatmap_md5: beatmap.beatmap_md5,
        beatmap_song_name: beatmap.song_name,
        beatmap_max_combo: beatmap.max_combo,
        mode: mode,
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
) -> ServiceResult<Option<NowPlayingState>> {
    match tillerino::fetch_last_np(ctx, session_id).await {
        Ok(last_np) => Ok(last_np),
        Err(e) => unexpected(e),
    }
}
