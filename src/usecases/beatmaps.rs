use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::models::beatmaps::{Beatmap, RankedStatus};
use crate::repositories::beatmaps;

const MIRRORS_BEATMAPSET_URL: &[(&str, &str)] = &[
    ("osu! (official servers)", "https://osu.ppy.sh/beatmapsets"),
    ("osu.direct (Chimu)", "https://osu.direct/beatmapsets"),
    (
        "Akatsuki (direct download)",
        "https://beatmaps.akatsuki.gg/api/d",
    ),
];

pub fn generate_mirror_links(beatmap_set_id: i32, song_name: &str) -> Vec<String> {
    MIRRORS_BEATMAPSET_URL
        .iter()
        .map(|(mirror_name, mirror_url)| {
            format!("{mirror_name}: [{mirror_url}/{beatmap_set_id} {song_name}]")
        })
        .collect()
}

pub async fn fetch_by_id<C: Context>(ctx: &C, map_id: i32) -> ServiceResult<Beatmap> {
    match beatmaps::fetch_by_id(ctx, map_id).await {
        Ok(beatmap) => Ok(Beatmap::from(beatmap)),
        Err(sqlx::Error::RowNotFound) => Err(AppError::BeatmapsNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn change_map_status<C: Context>(
    ctx: &C,
    map_id: i32,
    new_status: RankedStatus,
) -> ServiceResult<(Beatmap, RankedStatus)> {
    let mut map = beatmaps::fetch_by_id(ctx, map_id).await?;
    let previous_status = RankedStatus::from(map.ranked);
    if previous_status == new_status {
        return Ok((Beatmap::from(map), previous_status));
    }

    map.ranked = new_status as _;
    beatmaps::update_map_ranked_status(ctx, map_id, new_status as _).await?;
    beatmaps::publish_map_update(ctx, &map.beatmap_md5, new_status as _).await?;
    Ok((Beatmap::from(map), previous_status))
}

pub async fn change_set_status<C: Context>(
    ctx: &C,
    set_id: i32,
    new_status: RankedStatus,
) -> ServiceResult<impl Iterator<Item = (Beatmap, RankedStatus)>> {
    let maps = beatmaps::fetch_by_set_id(ctx, set_id).await?;
    beatmaps::update_set_ranked_status(ctx, set_id, new_status as _).await?;

    let beatmaps_to_update = maps.iter().filter(|map| map.ranked != new_status as i8);
    for map in beatmaps_to_update {
        beatmaps::publish_map_update(ctx, &map.beatmap_md5, new_status as _).await?;
    }

    let maps = maps.into_iter().map(move |map| {
        let mut map = Beatmap::from(map);
        let previous_status = map.ranked_status;
        map.ranked_status = new_status;
        (map, previous_status)
    });
    Ok(maps)
}
