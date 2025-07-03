use crate::common::context::Context;
use crate::entities::gamemodes::Gamemode;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;
use bancho_protocol::messages::MessageArgs;
use bancho_protocol::messages::client::CreateMatch;
use bancho_protocol::messages::server::MatchJoinSuccess;
use bancho_protocol::serde::BinarySerialize;
use bancho_protocol::structures::SlotStatus;

pub async fn handle<C: Context>(
    ctx: &C,
    session: &Session,
    mut args: CreateMatch<'_>,
) -> EventResult {
    let match_data = &mut args.match_data;
    let max_player_count = match_data
        .slots
        .iter()
        .filter(|slot| slot.status != SlotStatus::Locked)
        .count();
    let mp_match = multiplayer::create(
        ctx,
        session,
        match_data.name,
        match_data.password,
        match_data.beatmap_name,
        match_data.beatmap_md5,
        match_data.beatmap_id,
        Gamemode::from_mode_and_mods(match_data.mode, match_data.mods),
        max_player_count,
    )
    .await?;

    // little hack to reuse the match data
    match_data.id = mp_match.ingame_match_id();
    match_data.slots[0].status = SlotStatus::NotReady;
    match_data.slots[0].user_id = session.user_id as _;
    Ok(Some(MatchJoinSuccess(match_data).as_message().serialize()))
}
