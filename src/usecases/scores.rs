use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::entities::gamemodes::{CustomGamemode, Gamemode};
use crate::models::scores::{LastUserScore, ScoreStatus};
use crate::repositories::scores;
use bancho_protocol::structures::Mode;
use tracing::info;

pub async fn fetch_last_user_score<C: Context>(
    ctx: &C,
    user_id: i64,
    custom_gamemode: CustomGamemode,
) -> ServiceResult<LastUserScore> {
    match scores::fetch_last_user_score(ctx, user_id, custom_gamemode).await {
        Ok(Some(score)) => Ok(LastUserScore::from(score)),
        Ok(None) => Err(AppError::ScoresNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn remove_first_places<C: Context>(
    ctx: &C,
    user_id: i64,
    mode: Option<Mode>,
    custom_gamemode: Option<CustomGamemode>,
) -> ServiceResult<()> {
    let first_places = scores::fetch_first_places(ctx, user_id, mode, custom_gamemode).await?;
    for first_place in first_places {
        let mode = Mode::try_from(first_place.mode as u8)?;
        let custom_mode = CustomGamemode::try_from(first_place.rx as u8)?;
        let gamemode = Gamemode::from(mode, custom_mode);
        let new_first_place =
            scores::fetch_new_first_place(ctx, user_id, &first_place.beatmap_md5, gamemode).await?;
        match new_first_place {
            None => scores::remove_first_place(ctx, first_place.score_id).await?,
            Some(new) => {
                scores::transfer_first_place(ctx, first_place.score_id, new.score_id, new.user_id)
                    .await?
            }
        }
    }
    Ok(())
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

pub async fn overwrite_best_score_with_last_score<C: Context>(
    ctx: &C,
    user_id: i64,
    custom_gamemode: CustomGamemode,
) -> ServiceResult<String> {
    match scores::fetch_last_user_score(ctx, user_id, custom_gamemode).await? {
        Some(last_score) => {
            let best_score = scores::fetch_map_user_best_score(
                ctx,
                user_id,
                &last_score.beatmap_md5,
                custom_gamemode,
            )
            .await?;

            match best_score {
                Some(best_score) => {
                    if best_score.score_id == last_score.score_id {
                        return Err(AppError::ScoresNotFound);
                    }

                    scores::update_score_status(
                        ctx,
                        best_score.score_id,
                        ScoreStatus::Passed as _,
                        custom_gamemode,
                    )
                    .await?;
                    scores::update_score_status(
                        ctx,
                        last_score.score_id,
                        ScoreStatus::RankedScore as _,
                        custom_gamemode,
                    )
                    .await?;

                    Ok(todo!())
                }
                None => Err(AppError::ScoresNotFound),
            }
        }
        None => Err(AppError::ScoresNotFound),
    }
}
