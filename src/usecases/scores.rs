use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::entities::gamemodes::{CustomGamemode, Gamemode};
use crate::repositories::scores;
use bancho_protocol::structures::Mode;
use tracing::info;

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

/// This assumes the user is publicly visible
pub async fn recalculate_user_first_places<C: Context>(ctx: &C, user_id: i64) -> ServiceResult<()> {
    for custom_mode in CustomGamemode::all() {
        let scoring = custom_mode.scoring();
        info!("Recalculating first places for {custom_mode:?}");
        let user_scores = scores::fetch_user_scores(ctx, user_id, custom_mode).await?;

        for score in user_scores {
            let mode = Mode::try_from(score.mode as u8)?;
            let gamemode = Gamemode::from(mode, custom_mode);
            let current_first_place =
                scores::fetch_first_place(ctx, &score.beatmap_md5, gamemode).await?;

            let is_best_score = current_first_place.is_none()
                || current_first_place.is_some_and(|current_first_place| {
                    scoring.is_ranked_higher_than(&score, &current_first_place)
                });
            if is_best_score {
                info!("Updating first place score");
                scores::replace_first_place(
                    ctx,
                    score.score_id,
                    user_id,
                    &score.beatmap_md5,
                    gamemode,
                )
                .await?;
            }
        }
    }

    Ok(())
}
