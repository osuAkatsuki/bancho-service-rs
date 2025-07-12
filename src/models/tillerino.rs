use bancho_protocol::structures::{Mode, Mods};
use std::str::FromStr;

/// Represents the data sent by the osu! stable client when executing the /np command
#[derive(Debug)]
pub struct NowPlayingMessage<'a> {
    pub beatmap_id: i32,
    pub beatmap_set_id: i32,
    pub song_name: &'a str,
    pub mode: Mode,
    pub mods: Mods,
}

impl<'a> NowPlayingMessage<'a> {
    pub fn parse(content: &'a str) -> Option<Self> {
        const NP_ACTIONS: [&str; 4] = ["listening to", "playing", "editing", "watching"];

        let content = content
            .strip_prefix("\x01ACTION is ")?
            .strip_suffix("\x01")?;
        let map_link_markup_start = content.find(" [")?;
        let map_link_markup_end = content.rfind(']')?;
        if map_link_markup_end < map_link_markup_start {
            return None;
        }

        let action = &content[..map_link_markup_start];
        if !NP_ACTIONS.contains(&action) {
            return None;
        }

        let map_link_markup = &content[map_link_markup_start + 2..map_link_markup_end];
        let mode_and_mods = &content[map_link_markup_end + 1..].trim();

        // Parse map link and song name
        let mut map_link_parts = map_link_markup.splitn(2, ' ');
        let map_link = map_link_parts.next()?;
        let song_name = map_link_parts.next()?;

        // Get map id and set id from map link
        let mut map_link_parts = map_link.splitn(2, "/beatmapsets/");
        let _server_base_url = map_link_parts.next()?;
        let set_id_and_map_id = map_link_parts.next()?;

        let mut map_ids = set_id_and_map_id.splitn(2, '/');
        let beatmap_set_id_str = map_ids.next()?.trim_matches('#');
        let beatmap_id_str = map_ids.next()?;

        let beatmap_set_id = i32::from_str(beatmap_set_id_str).ok()?;
        let beatmap_id = i32::from_str(beatmap_id_str).ok()?;

        // Parse mode and mods
        let mut mode = Mode::Standard;
        let mut mods = Default::default();
        let mut mode_and_mods = mode_and_mods.split(' ');
        // The first entry might be a game mode
        if let Some(mode_or_mod) = mode_and_mods.next() {
            match mode_or_mod.starts_with('<') {
                true => {
                    let mode_str = mode_or_mod.trim_matches(['<', '>']);
                    mode = Mode::from_np(mode_str)?;
                }
                false => {
                    mods = Mods::from_np(mode_or_mod);
                }
            }

            while let Some(mod_str) = mode_and_mods.next() {
                mods |= Mods::from_np(mod_str);
            }
        }

        Some(Self {
            mode,
            mods,
            song_name,
            beatmap_id,
            beatmap_set_id,
        })
    }
}
