use bancho_protocol::structures::Mode;

use crate::entities::beatmaps::Beatmap as BeatmapEntity;

#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankedStatus {
    Pending = 0,
    Ranked = 2,
    Approved = 3,
    Qualified = 4,
    Loved = 5,
}

impl From<i8> for RankedStatus {
    fn from(value: i8) -> Self {
        match value {
            2 => Self::Ranked,
            3 => Self::Approved,
            4 => Self::Qualified,
            5 => Self::Loved,
            _ => Self::Pending,
        }
    }
}
pub struct Beatmap {
    pub beatmap_id: i32,
    pub beatmapset_id: i32,
    pub beatmap_md5: String,
    pub song_name: String,
    pub file_name: String,
    pub ar: f32,
    pub od: f32,
    pub mode: Mode,
    pub max_combo: i32,
    pub hit_length: i32,
    pub bpm: i32,
    pub ranked_status: RankedStatus,
    pub latest_update: i32,
    pub ranked_status_freezed: bool,
    pub playcount: i32,
    pub passcount: i32,
    pub rankedby: Option<i32>,
    pub rating: f64,
    pub bancho_ranked_status: Option<i16>,
}

impl From<BeatmapEntity> for Beatmap {
    fn from(entity: BeatmapEntity) -> Self {
        Self {
            beatmap_id: entity.beatmap_id,
            beatmapset_id: entity.beatmapset_id,
            beatmap_md5: entity.beatmap_md5,
            song_name: entity.song_name,
            file_name: entity.file_name,
            ar: entity.ar,
            od: entity.od,
            mode: Mode::try_from(entity.mode as u8).expect("Invalid mode"),
            max_combo: entity.max_combo,
            hit_length: entity.hit_length,
            bpm: entity.bpm,
            ranked_status: RankedStatus::from(entity.ranked),
            latest_update: entity.latest_update,
            ranked_status_freezed: entity.ranked_status_freezed,
            playcount: entity.playcount,
            passcount: entity.passcount,
            rankedby: entity.rankedby,
            rating: entity.rating,
            bancho_ranked_status: entity.bancho_ranked_status,
        }
    }
}
