use crate::common::context::Context;
use crate::common::error::ServiceResult;
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

pub async fn change_map_status<C: Context>(
    ctx: &C,
    map_id: i32,
    new_status: RankedStatus,
) -> ServiceResult<Beatmap> {
    let mut map = beatmaps::fetch_by_id(ctx, map_id).await?;
    if map.ranked == new_status as i8 {
        return Ok(Beatmap::from(map));
    }

    map.ranked = new_status as _;
    beatmaps::update_map_ranked_status(ctx, map_id, new_status as _).await?;
    beatmaps::publish_map_update(ctx, &map.beatmap_md5, new_status as _).await?;
    Ok(Beatmap::from(map))
}

pub async fn change_set_status<C: Context>(
    ctx: &C,
    set_id: i32,
    new_status: RankedStatus,
) -> ServiceResult<impl Iterator<Item = Beatmap>> {
    let mut maps = beatmaps::fetch_by_set_id(ctx, set_id).await?;
    beatmaps::update_set_ranked_status(ctx, set_id, new_status as _).await?;

    let beatmaps_to_update = maps.iter_mut().filter(|m| m.ranked != new_status as i8);
    for map in beatmaps_to_update {
        beatmaps::publish_map_update(ctx, &map.beatmap_md5, new_status as _).await?;
        map.ranked = new_status as _;
    }

    Ok(maps.into_iter().map(Beatmap::from))
}
