use crate::entities::scores::LastUserScore as LastUserScoreEntity;
use bancho_protocol::structures::{Mode, Mods};

#[repr(i8)]
pub enum ScoreStatus {
    Quit = 0,
    Failed = 1,
    Passed = 2,
    RankedScore = 3,
}

pub struct LastUserScore {
    pub score_id: i64,
    pub user_id: i64,
    pub mode: Mode,
    pub mods: Mods,
    pub score: i64,
    pub performance: f32,
    pub max_combo: i32,
    pub accuracy: f32,
    pub time: i32,

    pub beatmap_id: i32,
    pub beatmap_set_id: i32,
    pub beatmap_md5: String,
    pub song_name: String,
    pub beatmap_max_combo: i32,
}

impl From<LastUserScoreEntity> for LastUserScore {
    fn from(value: LastUserScoreEntity) -> Self {
        Self {
            score_id: value.score_id,
            user_id: value.user_id,
            mode: Mode::try_from(value.mode as u8).expect("invalid mode"),
            mods: Mods::from_bits_retain(value.mods as u32),
            score: value.score,
            performance: value.performance,
            max_combo: value.max_combo,
            accuracy: value.accuracy,
            time: value.time,
            beatmap_id: value.beatmap_id,
            beatmap_set_id: value.beatmap_set_id,
            beatmap_md5: value.beatmap_md5,
            song_name: value.song_name,
            beatmap_max_combo: value.beatmap_max_combo,
        }
    }
}
