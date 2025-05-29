use crate::entities::presences::Presence;
use bancho_protocol::concat_messages;
use bancho_protocol::messages::server::{UserPresence, UserStats};
use bancho_protocol::structures::{Action, Country, Mode, Mods, Privileges, UserAction};
use std::borrow::ToOwned;

pub const BOT_ID: i64 = 999;
pub const BOT_NAME: &str = "Aika";
const BOT_PRIVILEGES: Privileges =
    Privileges::from_bits_retain(Privileges::Player.bits() | Privileges::LeGuy.bits());

const USER_PRESENCE: UserPresence = UserPresence::new(
    BOT_ID as _,
    BOT_NAME,
    0,
    Country::SatelliteProvider,
    Mode::Standard,
    BOT_PRIVILEGES,
    69.69,
    133.7,
);

const USER_STATS: UserStats = UserStats {
    user_id: BOT_ID as _,
    action: UserAction {
        action: Action::Testing,
        info_text: "some stuff",
        beatmap_md5: "",
        mods: Mods::None,
        mode: Mode::Standard,
        beatmap_id: 0,
    },
    ranked_score: 0,
    accuracy: 4.2,
    plays: 1337,
    total_score: 1337420691337 * 24,
    global_rank: i32::MAX,
    performance: 1337,
};

pub fn presence() -> Presence {
    Presence {
        user_id: BOT_ID as _,
        username: BOT_NAME.to_owned(),
        utc_offset: 0,
        country_code: Country::SatelliteProvider.code().to_owned(),
        privileges: BOT_PRIVILEGES.bits() as _,
        latitude: 69.69,
        longitude: 133.7,

        action: USER_STATS.action.action as _,
        info_text: USER_STATS.action.info_text.to_owned(),
        beatmap_md5: USER_STATS.action.beatmap_md5.to_owned(),
        beatmap_id: USER_STATS.action.beatmap_id,
        mods: USER_STATS.action.mods.bits(),
        mode: USER_STATS.action.mode as _,

        ranked_score: USER_STATS.ranked_score as _,
        total_score: USER_STATS.total_score as _,
        accuracy: (USER_STATS.accuracy * 100.0) as _,
        playcount: USER_STATS.plays as _,
        performance: USER_STATS.performance as _,
        global_rank: USER_STATS.global_rank as _,
    }
}

pub fn user_panel() -> Vec<u8> {
    concat_messages!(USER_PRESENCE, USER_STATS)
}
