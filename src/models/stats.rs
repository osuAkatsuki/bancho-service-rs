use crate::entities::gamemodes::Gamemode;
use crate::entities::stats::Stats as Entity;

#[derive(Debug)]
pub struct Stats {
    pub user_id: i64,
    pub mode: Gamemode,
    pub ranked_score: u64,
    pub total_score: u64,
    pub playcount: u32,
    pub replays_watched: u32,
    pub total_hits: u32,
    pub level: u32,
    pub avg_accuracy: f64,
    pub pp: u32,
    pub playtime: u64,
    pub xh_count: u64,
    pub x_count: u64,
    pub sh_count: u64,
    pub s_count: u64,
    pub a_count: u64,
    pub b_count: u64,
    pub c_count: u64,
    pub d_count: u64,
    pub max_combo: u32,
    pub latest_pp_awarded: u64,
}

impl From<Entity> for Stats {
    fn from(value: Entity) -> Self {
        Self {
            user_id: value.user_id,
            mode: Gamemode::from_value(value.mode),
            ranked_score: value.ranked_score,
            total_score: value.total_score,
            playcount: value.playcount,
            replays_watched: value.replays_watched,
            total_hits: value.total_hits,
            level: value.level,
            avg_accuracy: value.avg_accuracy,
            pp: value.pp,
            playtime: value.playtime,
            xh_count: value.xh_count,
            x_count: value.x_count,
            sh_count: value.sh_count,
            s_count: value.s_count,
            a_count: value.a_count,
            b_count: value.b_count,
            c_count: value.c_count,
            d_count: value.d_count,
            max_combo: value.max_combo,
            latest_pp_awarded: value.latest_pp_awarded,
        }
    }
}
