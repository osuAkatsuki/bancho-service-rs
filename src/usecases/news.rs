use crate::common::context::Context;
use crate::common::error::{ServiceResult, unexpected};
use crate::common::website;
use crate::entities::news::NewsKind;
use crate::models::beatmaps::Beatmap;
use crate::repositories::news;

/// Writes the news item for `author_id` giving one difficulty the status `kind`.
pub async fn create_map_news<C: Context>(
    ctx: &C,
    kind: NewsKind,
    beatmap: &Beatmap,
    author_id: i64,
) -> ServiceResult<()> {
    let url = website::get_beatmap_link(beatmap.beatmap_id);
    match news::create(
        ctx,
        kind,
        &beatmap.song_name,
        None,
        Some(&url),
        Some(author_id),
        Some(beatmap.beatmapset_id),
        Some(beatmap.beatmap_id),
    )
    .await
    {
        Ok(()) => Ok(()),
        Err(e) => unexpected(e),
    }
}

/// Writes the news item for `author_id` giving a beatmap set the status `kind`.
/// `beatmap` is one of the set's difficulties whose status changed, and
/// `changed_count` is how many of them changed.
pub async fn create_set_news<C: Context>(
    ctx: &C,
    kind: NewsKind,
    beatmap: &Beatmap,
    changed_count: usize,
    author_id: i64,
) -> ServiceResult<()> {
    let title = song_name_without_version(&beatmap.song_name);
    let body = match changed_count {
        1 => "1 difficulty".to_owned(),
        n => format!("{n} difficulties"),
    };
    let url = website::get_beatmapset_link(beatmap.beatmapset_id);
    match news::create(
        ctx,
        kind,
        title,
        Some(&body),
        Some(&url),
        Some(author_id),
        Some(beatmap.beatmapset_id),
        None,
    )
    .await
    {
        Ok(()) => Ok(()),
        Err(e) => unexpected(e),
    }
}

/// Turns a difficulty's song name, "Artist - Title [Version]", into its set's
/// "Artist - Title" by removing the last " [...]" group. A name that does not
/// end in "]" carries no version and is returned unchanged.
fn song_name_without_version(song_name: &str) -> &str {
    match song_name.rfind(" [") {
        Some(version_start) if song_name.ends_with(']') => &song_name[..version_start],
        _ => song_name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_the_version() {
        assert_eq!(
            song_name_without_version("Artist - Title [Insane]"),
            "Artist - Title"
        );
    }

    #[test]
    fn keeps_a_name_without_a_version() {
        assert_eq!(
            song_name_without_version("Artist - Title"),
            "Artist - Title"
        );
    }

    #[test]
    fn removes_only_the_last_bracketed_group() {
        assert_eq!(
            song_name_without_version("Artist - Title [TV Size] [Hard]"),
            "Artist - Title [TV Size]"
        );
    }
}
