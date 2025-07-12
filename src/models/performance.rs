use crate::entities::tillerino::NowPlayingState;
use bancho_protocol::structures::Mods;
use std::str::FromStr;

pub struct PerformanceRequestArgs {
    pub beatmap_id: i32,
    pub beatmap_md5: String,
    pub beatmap_song_name: String,
    pub beatmap_max_combo: i32,

    pub mode: i32,
    pub mods: u32,
    pub extra: Option<PerformanceRequestExtra>,
}

impl PerformanceRequestArgs {
    pub fn from_extra(value: NowPlayingState, args: &str) -> anyhow::Result<Self> {
        let mut accuracy = None;
        let mut max_combo = None;
        let mut miss_count = None;
        let mut mods = None;
        let mut args = args.split(' ');
        while let Some(arg) = args.next() {
            match arg.strip_suffix('%') {
                Some(acc_str) => accuracy = Some(f32::from_str(acc_str)?),
                None => match arg.strip_suffix('x') {
                    Some(combo_str) => max_combo = Some(i32::from_str(combo_str)?),
                    None => match arg.strip_suffix('m').or(arg.strip_suffix("miss")) {
                        Some(misses_str) => miss_count = Some(i32::from_str(misses_str)?),
                        None => match Mods::from_str(arg) {
                            Ok(arg_mods) => mods = Some(arg_mods),
                            Err(_) => {}
                        },
                    },
                },
            };
        }
        let mods = mods.map(|m| m.bits()).unwrap_or(value.mods);

        Ok(Self {
            mods,
            beatmap_id: value.beatmap_id,
            beatmap_md5: value.beatmap_md5,
            beatmap_song_name: value.beatmap_song_name,
            beatmap_max_combo: value.beatmap_max_combo,
            mode: value.mode as _,
            extra: Some(PerformanceRequestExtra {
                accuracy,
                max_combo,
                miss_count,
            }),
        })
    }
}

impl From<NowPlayingState> for PerformanceRequestArgs {
    fn from(value: NowPlayingState) -> Self {
        Self {
            beatmap_id: value.beatmap_id,
            beatmap_md5: value.beatmap_md5,
            beatmap_song_name: value.beatmap_song_name,
            beatmap_max_combo: value.beatmap_max_combo,
            mode: value.mode as _,
            mods: value.mods,
            extra: None,
        }
    }
}

pub struct PerformanceRequestExtra {
    pub accuracy: Option<f32>,
    pub max_combo: Option<i32>,
    pub miss_count: Option<i32>,
}
