use crate::common::error::AppError;
use crate::entities::presences::Presence as Entity;
use crate::models::Gamemode;
use crate::models::stats::Stats;
use bancho_protocol::messages::server::UserStats;
use bancho_protocol::structures::{Action, Country, Mods, UserAction};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PresenceAction {
    pub action: Action,
    pub info_text: String,
    pub beatmap_md5: String,
    pub beatmap_id: i32,
    pub mods: Mods,
    pub mode: Gamemode,
}

impl PresenceAction {
    pub fn from(action: UserAction<'_>) -> Self {
        Self {
            action: action.action,
            info_text: action.info_text.to_string(),
            beatmap_md5: action.beatmap_md5.to_string(),
            beatmap_id: action.beatmap_id,
            mods: action.mods,
            mode: Gamemode::from_mode_and_mods(action.mode, action.mods),
        }
    }

    pub fn to_bancho(&self) -> UserAction<'_> {
        UserAction {
            action: self.action,
            info_text: &self.info_text,
            beatmap_md5: &self.beatmap_md5,
            beatmap_id: self.beatmap_id,
            mods: self.mods,
            mode: self.mode.to_bancho(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PresenceStats {
    pub ranked_score: u64,
    pub total_score: u64,
    pub accuracy: f64,
    pub playcount: u32,
    pub performance: u32,
    pub global_rank: usize,
}

impl PresenceStats {
    pub fn from(stats: Stats, global_rank: usize) -> Self {
        Self {
            ranked_score: stats.ranked_score,
            total_score: stats.total_score,
            accuracy: stats.avg_accuracy,
            playcount: stats.playcount,
            performance: stats.pp,
            global_rank,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LocationInformation {
    pub country: Country,
    pub longitude: f32,
    pub latitude: f32,
    pub utc_offset: i8,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Presence {
    pub user_id: i64,
    pub action: PresenceAction,
    pub stats: PresenceStats,
    pub location: LocationInformation,
}

impl Presence {
    pub fn to_bancho(&self) -> UserStats<'_> {
        UserStats {
            user_id: self.user_id as _,
            action: self.action.to_bancho(),
            ranked_score: self.stats.ranked_score as _,
            total_score: self.stats.total_score as _,
            accuracy: (self.stats.accuracy / 100.0) as _,
            plays: self.stats.playcount as _,
            performance: self.stats.performance as _,
            global_rank: self.stats.global_rank as _,
        }
    }
}

impl Into<Entity> for Presence {
    fn into(self) -> Entity {
        Entity {
            user_id: self.user_id,
            action: self.action.action as _,
            info_text: self.action.info_text,
            beatmap_md5: self.action.beatmap_md5,
            beatmap_id: self.action.beatmap_id,
            mods: self.action.mods.bits(),
            mode: self.action.mode as _,
            ranked_score: self.stats.ranked_score,
            total_score: self.stats.total_score,
            accuracy: self.stats.accuracy,
            playcount: self.stats.playcount,
            performance: self.stats.performance,
            global_rank: self.stats.global_rank,

            country_code: self.location.country.code().to_owned(),
            longitude: self.location.longitude,
            latitude: self.location.latitude,
            utc_offset: self.location.utc_offset,
        }
    }
}

impl TryFrom<Entity> for Presence {
    type Error = AppError;

    fn try_from(value: Entity) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: value.user_id,
            action: PresenceAction {
                action: Action::try_from(value.action)?,
                info_text: value.info_text,
                beatmap_md5: value.beatmap_md5,
                beatmap_id: value.beatmap_id,
                mods: Mods::from_bits_retain(value.mods),
                mode: Gamemode::try_from(value.mode)?,
            },
            stats: PresenceStats {
                ranked_score: value.ranked_score,
                total_score: value.total_score,
                accuracy: value.accuracy,
                playcount: value.playcount,
                performance: value.performance,
                global_rank: value.global_rank,
            },
            location: LocationInformation {
                country: Country::try_from_iso3166_2(&value.country_code)?,
                longitude: value.longitude,
                latitude: value.latitude,
                utc_offset: value.utc_offset,
            },
        })
    }
}
