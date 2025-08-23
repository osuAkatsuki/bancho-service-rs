use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::common::state::AppState;
use crate::entities::bot;
use crate::entities::channels::ChannelName;
use crate::entities::gamemodes::Gamemode;
use crate::entities::match_events::MatchEventType;
use crate::entities::multiplayer::MultiplayerMatchSlot as SlotEntity;
use crate::entities::sessions::SessionIdentity;
use crate::models::multiplayer::MatchSlotExt;
use crate::models::multiplayer::{MultiplayerMatch, MultiplayerMatchSlot, MultiplayerMatchSlots};
use crate::models::presences::PresenceStats;
use crate::models::sessions::Session;
use crate::repositories::multiplayer::TimerType;
use crate::repositories::streams::StreamName;
use crate::repositories::{match_games, multiplayer};
use crate::usecases::{channels, match_events, presences, sessions, stats, streams};
use bancho_protocol::concat_messages;
use bancho_protocol::messages::MessageArgs;
use bancho_protocol::messages::server::{
    Alert, ChatMessage, MatchAborted, MatchAllPlayersLoaded, MatchComplete, MatchCreated,
    MatchDisposed, MatchJoinFailed, MatchPlayerFailed, MatchPlayerSkipped, MatchSkip, MatchStart,
    MatchUpdate,
};
use bancho_protocol::serde::BinarySerialize;
use bancho_protocol::structures::{IrcMessage, Match, MatchTeam, Mods, SlotStatus};
use std::time::Duration;
use tracing::error;
use uuid::Uuid;

pub async fn create<C: Context>(
    ctx: &C,
    host_session: &Session,
    name: &str,
    password: &str,
    beatmap_name: &str,
    beatmap_md5: &str,
    beatmap_id: i32,
    mode: Gamemode,
    max_player_count: usize,
) -> ServiceResult<MultiplayerMatch> {
    if let Some(match_id) =
        multiplayer::fetch_session_match_id(ctx, host_session.session_id).await?
    {
        leave(ctx, host_session.identity(), Some(match_id)).await?;
    }

    let (mp_match, slots) = multiplayer::create(
        ctx,
        host_session.identity(),
        name,
        password,
        beatmap_name,
        beatmap_md5,
        beatmap_id,
        mode as _,
        max_player_count,
    )
    .await?;

    match_events::create(
        ctx,
        mp_match.match_id,
        MatchEventType::MatchCreated,
        Some(mp_match.host_user_id),
        None,
    )
    .await?;

    let mp_match = MultiplayerMatch::try_from(mp_match)?;
    let slots = MultiplayerMatchSlot::from(slots);

    streams::leave(ctx, host_session.session_id, StreamName::Lobby).await?;
    streams::join(
        ctx,
        host_session.session_id,
        StreamName::Multiplayer(mp_match.match_id),
    )
    .await?;
    channels::join(
        ctx,
        host_session,
        ChannelName::Multiplayer(mp_match.match_id),
    )
    .await?;

    let match_notification = mp_match.as_bancho(slots);
    streams::broadcast_message(
        ctx,
        StreamName::Lobby,
        MatchCreated(&match_notification),
        None,
        None,
    )
    .await?;
    Ok(mp_match)
}

pub async fn fetch_session_match_id<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> ServiceResult<Option<i64>> {
    match multiplayer::fetch_session_match_id(ctx, session_id).await {
        Ok(match_id) => Ok(match_id),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_one<C: Context>(ctx: &C, match_id: i64) -> ServiceResult<MultiplayerMatch> {
    match multiplayer::fetch_one(ctx, match_id).await {
        Ok(Some(mp_match)) => Ok(MultiplayerMatch::try_from(mp_match)?),
        Ok(None) => Err(AppError::MultiplayerNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_all<C: Context>(ctx: &C) -> ServiceResult<Vec<MultiplayerMatch>> {
    match multiplayer::fetch_all(ctx).await {
        Ok(matches) => matches.map(MultiplayerMatch::try_from).collect(),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_all_with_slots<C: Context>(
    ctx: &C,
) -> ServiceResult<Vec<(MultiplayerMatch, MultiplayerMatchSlots)>> {
    let matches = fetch_all(ctx).await?;
    let mut result = vec![];
    for mp_match in matches {
        let slots = fetch_all_slots(ctx, mp_match.match_id).await?;
        result.push((mp_match, slots));
    }
    Ok(result)
}

pub async fn fetch_all_slots<C: Context>(
    ctx: &C,
    match_id: i64,
) -> ServiceResult<MultiplayerMatchSlots> {
    match multiplayer::fetch_all_slots(ctx, match_id).await {
        Ok(slots) => Ok(MultiplayerMatchSlot::from(slots)),
        Err(e) => unexpected(e),
    }
}

pub async fn update<C: Context>(
    ctx: &C,
    updated_match: MultiplayerMatch,
) -> ServiceResult<MultiplayerMatch> {
    let _ = fetch_one(ctx, updated_match.match_id).await?;
    multiplayer::update(ctx, updated_match.as_entity(), false).await?;

    let slots = fetch_all_slots(ctx, updated_match.match_id).await?;
    broadcast_update(ctx, &updated_match, slots).await?;
    Ok(updated_match)
}

pub async fn update_all_slots<C: Context>(
    ctx: &C,
    match_id: i64,
    slots: MultiplayerMatchSlots,
) -> ServiceResult<MultiplayerMatchSlots> {
    multiplayer::update_all_slots(ctx, match_id, slots.as_entity()).await?;
    Ok(slots)
}

fn ingame_match_id(match_id: i64) -> i32 {
    (match_id & 0xFFFF) as _
}

pub async fn delete<C: Context>(ctx: &C, match_id: i64) -> ServiceResult<()> {
    multiplayer::delete(ctx, match_id).await?;
    streams::clear_stream(ctx, StreamName::Multiplayer(match_id)).await?;
    streams::clear_stream(ctx, StreamName::Multiplaying(match_id)).await?;
    match_events::create(ctx, match_id, MatchEventType::MatchDisbanded, None, None).await?;
    streams::broadcast_message(
        ctx,
        StreamName::Lobby,
        MatchDisposed {
            match_id: ingame_match_id(match_id),
        },
        None,
        None,
    )
    .await?;
    Ok(())
}

pub async fn join<C: Context>(
    ctx: &C,
    session: &Session,
    match_id: i64,
    password: &str,
) -> ServiceResult<(MultiplayerMatch, MultiplayerMatchSlots)> {
    if let Some(match_id) = multiplayer::fetch_session_match_id(ctx, session.session_id).await? {
        leave(ctx, session.identity(), Some(match_id)).await?;
    }

    let mp_match = fetch_one(ctx, match_id).await?;
    if mp_match.password != password {
        return Err(AppError::MultiplayerInvalidPassword);
    }

    streams::leave(ctx, session.session_id, StreamName::Lobby).await?;
    let slots = multiplayer::join(ctx, session.identity(), mp_match.match_id)
        .await?
        .map(MultiplayerMatchSlot::from)
        .ok_or(AppError::MultiplayerMatchFull)?;

    let _ = match_events::create(
        ctx,
        match_id,
        MatchEventType::MatchUserJoined,
        Some(session.user_id),
        None,
    )
    .await;

    streams::join(
        ctx,
        session.session_id,
        StreamName::Multiplayer(mp_match.match_id),
    )
    .await?;
    channels::join(ctx, session, ChannelName::Multiplayer(mp_match.match_id)).await?;

    broadcast_update(ctx, &mp_match, slots).await?;
    Ok((mp_match, slots))
}

pub async fn leave<C: Context>(
    ctx: &C,
    session: SessionIdentity,
    match_id: Option<i64>,
) -> ServiceResult<()> {
    let match_id = match match_id {
        Some(match_id) => match_id,
        None => match multiplayer::fetch_session_match_id(ctx, session.session_id).await? {
            Some(match_id) => match_id,
            None => return Ok(()),
        },
    };

    let mut mp_match = fetch_one(ctx, match_id).await?;
    let (user_count, slots) =
        match multiplayer::leave(ctx, session.session_id, mp_match.match_id).await? {
            Some((user_count, slots)) => (user_count, MultiplayerMatchSlot::from(slots)),
            None => return Ok(()),
        };

    let _ = match_events::create(
        ctx,
        match_id,
        MatchEventType::MatchUserLeft,
        Some(session.user_id),
        None,
    )
    .await;

    streams::leave(
        ctx,
        session.session_id,
        StreamName::Multiplayer(mp_match.match_id),
    )
    .await?;
    streams::leave(
        ctx,
        session.session_id,
        StreamName::Multiplaying(mp_match.match_id),
    )
    .await?;
    channels::leave(
        ctx,
        session.session_id,
        ChannelName::Multiplayer(mp_match.match_id),
    )
    .await?;

    if user_count == 0 {
        delete(ctx, match_id).await?;
    } else {
        if mp_match.host_user_id == session.user_id {
            match slots.iter().filter_map(|slot| slot.user).next() {
                Some(next_host) => {
                    mp_match.host_user_id = next_host.user_id;
                    multiplayer::update(ctx, mp_match.as_entity(), false).await?;
                    let _ = match_events::create(
                        ctx,
                        mp_match.match_id,
                        MatchEventType::MatchHostAssignment,
                        Some(next_host.user_id),
                        None,
                    )
                    .await;
                }
                None => {}
            };
        }

        broadcast_update(ctx, &mp_match, slots).await?;
    }
    Ok(())
}

fn split_mods(mods: Mods) -> (Mods, Mods) {
    let match_mods = mods.intersection(Mods::Halftime | Mods::Doubletime | Mods::Nightcore);
    (mods & !match_mods, match_mods)
}

// TODO: get rid of this
pub async fn update_bancho<C: Context>(
    ctx: &C,
    match_id: i64,
    args: Match<'_>,
    check_host: Option<i64>,
) -> ServiceResult<MultiplayerMatch> {
    let mut mp_match = fetch_one(ctx, match_id).await?;
    if let Some(check_host) = check_host
        && mp_match.host_user_id != check_host
        && !is_referee(ctx, match_id, check_host).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    let update_name = mp_match.name != args.name;
    let update_private = mp_match.password.is_empty() != args.password.is_empty();
    if mp_match.password != args.password {
        mp_match.password = args.password.to_string();
    }
    if update_name {
        mp_match.name = args.name.to_string();
    }
    if mp_match.beatmap_name != args.beatmap_name {
        mp_match.beatmap_name = args.beatmap_name.to_string();
        mp_match.beatmap_md5 = args.beatmap_md5.to_string();
    }
    mp_match.beatmap_id = args.beatmap_id;

    let match_mods = mp_match.mods;
    let mut slots = multiplayer::fetch_all_slots(ctx, mp_match.match_id).await?;
    let new_mode = Gamemode::from_mode_and_mods(args.mode, match_mods);
    if new_mode != mp_match.mode {
        // Update stats for all match members when mode changes
        let match_member_ids = slots.iter().filter_map(|slot| match slot.user {
            None => None,
            Some(user) => Some(user.user_id),
        });
        update_match_members_presences(ctx, match_member_ids, new_mode).await?;
    }

    mp_match.mode = new_mode;
    mp_match.win_condition = args.win_condition as _;
    mp_match.team_type = args.team_type as _;
    mp_match.random_seed = args.random_seed;

    let freemod_changed = mp_match.freemod_enabled != args.freemod_enabled;
    if freemod_changed {
        mp_match.freemod_enabled = args.freemod_enabled;
        if mp_match.freemod_enabled {
            let (slot_mods, match_mods) = split_mods(match_mods);
            mp_match.mods = match_mods;
            slots
                .iter_mut()
                .filter(|slot| slot.user.is_some())
                .for_each(|slot| slot.mods = slot_mods.bits());
            multiplayer::update_all_slots(ctx, mp_match.match_id, slots).await?;
        }
    }

    broadcast_update(ctx, &mp_match, MultiplayerMatchSlot::from(slots)).await?;
    let mp_match = multiplayer::update(ctx, mp_match.into(), update_name || update_private).await?;
    Ok(MultiplayerMatch::try_from(mp_match)?)
}

pub async fn fetch_user_slot<C: Context>(
    ctx: &C,
    match_id: i64,
    user_id: i64,
) -> ServiceResult<(usize, MultiplayerMatchSlot)> {
    let slots = fetch_all_slots(ctx, match_id).await?;
    let slot = slots
        .into_iter()
        .enumerate()
        .find(|(_, slot)| {
            slot.user
                .is_some_and(|slot_user| slot_user.user_id == user_id)
        })
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    Ok(slot)
}

pub async fn fetch_session_slot<C: Context>(
    ctx: &C,
    match_id: i64,
    session_id: Uuid,
) -> ServiceResult<(usize, MultiplayerMatchSlot)> {
    let slots = fetch_all_slots(ctx, match_id).await?;
    let slot = slots
        .into_iter()
        .enumerate()
        .find(|(_, slot)| {
            slot.user
                .is_some_and(|slot_user| slot_user.session_id == session_id)
        })
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    Ok(slot)
}

pub async fn transfer_host_to_slot<C: Context>(
    ctx: &C,
    match_id: i64,
    slot_id: usize,
    check_host: Option<i64>,
) -> ServiceResult<()> {
    if slot_id > 15 {
        return Err(AppError::MultiplayerSlotNotFound);
    }
    let mut mp_match = fetch_one(ctx, match_id).await?;
    if let Some(check_host) = check_host
        && mp_match.host_user_id != check_host
        && !is_referee(ctx, match_id, check_host).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    let slots = fetch_all_slots(ctx, mp_match.match_id).await?;
    let slot_user_id = slots[slot_id]
        .user
        .ok_or(AppError::MultiplayerSlotNotFound)?
        .user_id;
    mp_match.host_user_id = slot_user_id;
    multiplayer::update(ctx, mp_match.as_entity(), false).await?;
    broadcast_update(ctx, &mp_match, slots).await?;

    match_events::create(
        ctx,
        mp_match.match_id,
        MatchEventType::MatchHostAssignment,
        Some(slot_user_id),
        None,
    )
    .await?;

    Ok(())
}

pub async fn transfer_host_to_user<C: Context>(
    ctx: &C,
    match_id: i64,
    user_id: i64,
    check_referee: Option<i64>,
) -> ServiceResult<()> {
    let mut mp_match = fetch_one(ctx, match_id).await?;
    if let Some(check_referee_user_id) = check_referee
        && mp_match.host_user_id != check_referee_user_id
        && !is_referee(ctx, match_id, check_referee_user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    let slots = fetch_all_slots(ctx, mp_match.match_id).await?;
    let _slot = slots
        .iter()
        .find(|slot| {
            slot.user
                .is_some_and(|slot_user| slot_user.user_id == user_id)
        })
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    mp_match.host_user_id = user_id;
    multiplayer::update(ctx, mp_match.as_entity(), false).await?;
    broadcast_update(ctx, &mp_match, slots).await?;

    match_events::create(
        ctx,
        mp_match.match_id,
        MatchEventType::MatchHostAssignment,
        Some(user_id),
        None,
    )
    .await?;

    Ok(())
}

pub async fn clear_host<C: Context>(ctx: &C, match_id: i64) -> ServiceResult<()> {
    let mut mp_match = match multiplayer::fetch_one(ctx, match_id).await? {
        Some(mp_match) => mp_match,
        None => return Err(AppError::MultiplayerNotFound),
    };
    let slots = fetch_all_slots(ctx, mp_match.match_id).await?;
    mp_match.host_user_id = 0;
    match multiplayer::update(ctx, mp_match, false).await {
        Ok(mp_match) => {
            let mp_match = MultiplayerMatch::try_from(mp_match)?;
            broadcast_update(ctx, &mp_match, slots).await?;
            Ok(())
        }
        Err(e) => unexpected(e),
    }
}

pub async fn swap_slots<C: Context>(
    ctx: &C,
    match_id: i64,
    from_slot_id: usize,
    to_slot_id: usize,
) -> ServiceResult<()> {
    let mp_match = fetch_one(ctx, match_id).await?;
    let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;

    let from_slot = slots[from_slot_id];
    let to_slot = slots[to_slot_id];

    slots[from_slot_id] = to_slot;
    slots[to_slot_id] = from_slot;

    multiplayer::update_slots(
        ctx,
        match_id,
        [(from_slot_id, to_slot), (to_slot_id, from_slot)],
    )
    .await?;
    broadcast_update(ctx, &mp_match, MultiplayerMatchSlot::from(slots)).await?;
    Ok(())
}

pub async fn swap_session_slots<C: Context>(
    ctx: &C,
    match_id: i64,
    target_slot_id: usize,
    session_id: Uuid,
) -> ServiceResult<()> {
    let mp_match = fetch_one(ctx, match_id).await?;
    let mut slots = fetch_all_slots(ctx, match_id).await?;

    let (user_slot_id, user_slot) = slots
        .iter()
        .enumerate()
        .find(|(_, slot)| {
            slot.user
                .is_some_and(|slot_user| slot_user.session_id == session_id)
        })
        .map(|(id, slot)| (id, *slot))
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let target_slot = slots[target_slot_id];
    slots[target_slot_id] = user_slot;
    slots[user_slot_id] = target_slot;

    multiplayer::update_slots(
        ctx,
        match_id,
        [
            (user_slot_id, target_slot.into()),
            (target_slot_id, user_slot.into()),
        ],
    )
    .await?;
    broadcast_update(ctx, &mp_match, slots).await?;
    Ok(())
}

pub async fn set_session_slot_status<C: Context>(
    ctx: &C,
    match_id: i64,
    session_id: Uuid,
    status: SlotStatus,
    check_host: Option<i64>,
) -> ServiceResult<()> {
    let (slot_id, _) = fetch_session_slot(ctx, match_id, session_id).await?;
    set_slot_status(ctx, match_id, slot_id, status, check_host).await
}

pub async fn set_slot_status<C: Context>(
    ctx: &C,
    match_id: i64,
    slot_id: usize,
    status: SlotStatus,
    check_host: Option<i64>,
) -> ServiceResult<()> {
    if slot_id > 15 {
        return Err(AppError::MultiplayerSlotNotFound);
    }

    let mp_match = fetch_one(ctx, match_id).await?;
    if let Some(check_host) = check_host
        && check_host != mp_match.host_user_id
        && !is_referee(ctx, match_id, check_host).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    let slot = multiplayer::fetch_slot(ctx, match_id, slot_id).await?;
    let mut slot = slot.ok_or(AppError::MultiplayerSlotNotFound)?;
    let slot_locked = slot.status == SlotStatus::Locked.bits();
    let locking_slot = status == SlotStatus::Locked;
    if let Some(slot_user) = slot.user
        && locking_slot
    {
        slot.clear();
        // kick the user
        if let Ok(session) = sessions::fetch_one(ctx, slot_user.session_id).await {
            leave(ctx, session.identity(), Some(match_id)).await?;
            let notification = concat_messages!(
                MatchJoinFailed,
                Alert {
                    message: "You have been kicked out of the match!",
                },
            );
            streams::broadcast_data(
                ctx,
                StreamName::User(session.session_id),
                &notification,
                None,
                None,
            )
            .await?;
        }
    }
    if slot_locked && locking_slot {
        slot.status = SlotStatus::Empty.bits();
    } else {
        slot.status = status.bits();
    }
    multiplayer::update_slot(ctx, match_id, slot_id, slot).await?;

    let slots = fetch_all_slots(ctx, match_id).await?;
    broadcast_update(ctx, &mp_match, slots).await?;
    Ok(())
}

pub async fn set_user_team<C: Context>(
    ctx: &C,
    match_id: i64,
    user_id: i64,
    team: MatchTeam,
) -> ServiceResult<()> {
    let mp_match = fetch_one(ctx, match_id).await?;
    let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    let (slot_id, slot) = slots
        .iter_mut()
        .enumerate()
        .find(|(_, slot)| {
            slot.user
                .is_some_and(|slot_user| slot_user.user_id == user_id)
        })
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    slot.team = team as u8;
    multiplayer::update_slot(ctx, match_id, slot_id, *slot).await?;
    broadcast_update(ctx, &mp_match, MultiplayerMatchSlot::from(slots)).await?;
    Ok(())
}

pub async fn switch_teams<C: Context>(
    ctx: &C,
    match_id: i64,
    session_id: Uuid,
) -> ServiceResult<()> {
    let mp_match = fetch_one(ctx, match_id).await?;
    let mut slots = fetch_all_slots(ctx, match_id).await?;
    let (slot_id, slot) = slots
        .iter_mut()
        .enumerate()
        .find(|(_, slot)| {
            slot.user
                .is_some_and(|slot_user| slot_user.session_id == session_id)
        })
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    slot.team = match slot.team {
        MatchTeam::None => MatchTeam::Blue,
        MatchTeam::Blue => MatchTeam::Red,
        MatchTeam::Red => MatchTeam::Blue,
    };
    multiplayer::update_slot(ctx, match_id, slot_id, slot.as_entity()).await?;
    broadcast_update(ctx, &mp_match, slots).await?;
    Ok(())
}

pub async fn start_game<C: Context>(
    ctx: &C,
    match_id: i64,
    check_host: Option<i64>,
) -> ServiceResult<()> {
    let mut mp_match = multiplayer::fetch_one(ctx, match_id)
        .await?
        .ok_or(AppError::MultiplayerNotFound)?;
    if let Some(check_host) = check_host
        && check_host != mp_match.host_user_id
        && !is_referee(ctx, match_id, check_host).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }
    mp_match.in_progress = true;
    let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    for slot in &mut slots {
        let slot_status = SlotStatus::from_bits_retain(slot.status);
        if let Some(slot_user) = slot.user {
            if slot_status.intersects(SlotStatus::Ready | SlotStatus::NotReady) {
                slot.status = SlotStatus::Playing.bits();
                streams::join(
                    ctx,
                    slot_user.session_id,
                    StreamName::Multiplaying(match_id),
                )
                .await?;
            } else {
                streams::leave(
                    ctx,
                    slot_user.session_id,
                    StreamName::Multiplaying(match_id),
                )
                .await?;
            }
        }
    }

    let game_id = match_games::create(
        ctx,
        match_id,
        mp_match.beatmap_id,
        mp_match.mode,
        mp_match.mods,
        mp_match.win_condition,
        mp_match.team_type,
    )
    .await?;
    let _ = match_events::create(
        ctx,
        match_id,
        MatchEventType::MatchGamePlaythrough,
        None,
        Some(game_id),
    )
    .await;
    mp_match.last_game_id = Some(game_id);
    let mp_match = multiplayer::update(ctx, mp_match, false).await?;
    multiplayer::update_all_slots(ctx, match_id, slots).await?;

    let mp_match = MultiplayerMatch::try_from(mp_match)?;
    let slots = MultiplayerMatchSlot::from(slots);
    let bancho_match = mp_match.as_bancho(slots);
    streams::broadcast_message(
        ctx,
        StreamName::Lobby,
        MatchUpdate(&bancho_match),
        None,
        None,
    )
    .await?;
    streams::broadcast_message(
        ctx,
        StreamName::Multiplayer(match_id),
        MatchUpdate(&bancho_match),
        None,
        None,
    )
    .await?;
    streams::broadcast_message(
        ctx,
        StreamName::Multiplaying(match_id),
        MatchStart(&bancho_match),
        None,
        None,
    )
    .await?;
    Ok(())
}

pub async fn end_game<C: Context>(ctx: &C, match_id: i64) -> ServiceResult<()> {
    streams::broadcast_message(
        ctx,
        StreamName::Multiplaying(match_id),
        MatchComplete,
        None,
        None,
    )
    .await?;
    match_games::game_ended(ctx, match_id).await?;

    let mut mp_match = multiplayer::fetch_one(ctx, match_id)
        .await?
        .ok_or(AppError::MultiplayerNotFound)?;
    let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    mp_match.in_progress = false;
    slots.iter_mut().for_each(|slot| {
        if slot.user.is_some() {
            slot.status = SlotStatus::NotReady.bits();
        }
    });

    let mp_match = multiplayer::update(ctx, mp_match, false).await?;
    multiplayer::update_all_slots(ctx, match_id, slots).await?;

    let mp_match = MultiplayerMatch::try_from(mp_match)?;
    let slots = MultiplayerMatchSlot::from(slots);
    broadcast_update(ctx, &mp_match, slots).await?;
    Ok(())
}

pub async fn player_loaded<C: Context>(ctx: &C, session: &Session) -> ServiceResult<bool> {
    let match_id = fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    let (all_loaded, _) =
        change_playing_state(ctx, match_id, session.session_id, |slot| &mut slot.loaded).await?;
    if all_loaded {
        streams::broadcast_message(
            ctx,
            StreamName::Multiplaying(match_id),
            MatchAllPlayersLoaded,
            None,
            None,
        )
        .await?;
    }
    Ok(all_loaded)
}

pub async fn skip_requested<C: Context>(ctx: &C, session: &Session) -> ServiceResult<bool> {
    let match_id = fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    let (all_skipped, slot_id) =
        change_playing_state(ctx, match_id, session.session_id, |slot| &mut slot.skipped).await?;
    let skip_notification = match all_skipped {
        true => MatchSkip.as_message().serialize(),
        false => MatchPlayerSkipped {
            slot_id: slot_id as _,
        }
        .as_message()
        .serialize(),
    };
    streams::broadcast_data(
        ctx,
        StreamName::Multiplaying(match_id),
        &skip_notification,
        None,
        None,
    )
    .await?;
    Ok(all_skipped)
}

pub async fn player_failed<C: Context>(ctx: &C, session: &Session) -> ServiceResult<bool> {
    let match_id = fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    let (all_failed, slot_id) =
        change_playing_state(ctx, match_id, session.session_id, |slot| &mut slot.failed).await?;
    streams::broadcast_message(
        ctx,
        StreamName::Multiplaying(match_id),
        MatchPlayerFailed {
            slot_id: slot_id as _,
        },
        None,
        None,
    )
    .await?;
    Ok(all_failed)
}

pub async fn player_completed<C: Context>(ctx: &C, session: &Session) -> ServiceResult<bool> {
    let match_id = fetch_session_match_id(ctx, session.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    let (all_completed, _slot_id) =
        change_playing_state(ctx, match_id, session.session_id, |slot| {
            &mut slot.completed
        })
        .await?;
    if all_completed {
        end_game(ctx, match_id).await?;
    }

    Ok(all_completed)
}

pub async fn change_mods<C: Context>(
    ctx: &C,
    match_id: i64,
    mods: Mods,
    slot_user: Option<SessionIdentity>,
) -> ServiceResult<()> {
    let mut mp_match = multiplayer::fetch_one(ctx, match_id)
        .await?
        .ok_or(AppError::MultiplayerNotFound)?;
    // if a user is making the request, check if they are the host or whether freemod is enabled
    if let Some(slot_user) = slot_user
        && mp_match.host_user_id != slot_user.user_id
        && !mp_match.freemod_enabled
        && !is_referee(ctx, match_id, slot_user.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    let match_mode = Gamemode::try_from(mp_match.mode)?;
    let match_mode_vn = match_mode.as_bancho();
    let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    if mp_match.freemod_enabled {
        let (new_slot_mods, match_mods) = split_mods(mods);
        mp_match.mods = match_mods.bits();

        // if a user is making the request, only update their slot
        if let Some(slot_user) = slot_user {
            let (slot_id, slot) = slots
                .iter_mut()
                .enumerate()
                .find(|(_, slot)| {
                    slot.user
                        .is_some_and(|su| su.session_id == slot_user.session_id)
                })
                .ok_or(AppError::MultiplayerUserNotInMatch)?;

            let new_mods_bits = new_slot_mods.bits();
            let old_slot_mods = slot.mods;
            slot.mods = new_mods_bits;
            multiplayer::update_slot(ctx, match_id, slot_id, slot.clone()).await?;

            // Update the players' presence
            let affected_mods = Mods::from_bits_retain(old_slot_mods ^ new_mods_bits);
            let reload_stats = affected_mods.has_any(Mods::Relax | Mods::Autopilot);
            if reload_stats {
                let new_mode = Gamemode::from_mode_and_mods(match_mode_vn, new_slot_mods);
                update_match_members_presences(ctx, vec![slot_user.user_id].into_iter(), new_mode)
                    .await?;
            }
        } else {
            let new_mods_bits = new_slot_mods.bits();
            let new_mode = Gamemode::from_mode_and_mods(match_mode_vn, new_slot_mods);
            mp_match.mode = new_mode as _;

            let update_user_ids = slots.iter_mut().filter_map(|slot| match slot.user {
                Some(slot_user) => {
                    let old_slot_mods = slot.mods;
                    slot.mods = new_mods_bits;

                    let affected_mods = Mods::from_bits_retain(old_slot_mods ^ new_mods_bits);
                    match affected_mods.has_any(Mods::Relax | Mods::Autopilot) {
                        true => Some(slot_user.user_id),
                        false => None,
                    }
                }
                None => None,
            });

            update_match_members_presences(ctx, update_user_ids, new_mode).await?;
            multiplayer::update_all_slots(ctx, match_id, slots).await?;
        }
    } else {
        let new_mode = Gamemode::from_mode_and_mods(match_mode_vn, mods);
        mp_match.mode = new_mode as _;
        mp_match.mods = mods.bits();

        if new_mode != match_mode {
            // Update stats for all match members when mode changes
            let match_member_ids = slots.iter().filter_map(|slot| match slot.user {
                None => None,
                Some(user) => Some(user.user_id),
            });
            update_match_members_presences(ctx, match_member_ids, new_mode).await?;
        }
    }

    let mp_match = multiplayer::update(ctx, mp_match, false).await?;
    let mp_match = MultiplayerMatch::try_from(mp_match)?;
    let slots = MultiplayerMatchSlot::from(slots);
    broadcast_update(ctx, &mp_match, slots).await?;
    Ok(())
}

pub async fn is_referee<C: Context>(ctx: &C, match_id: i64, user_id: i64) -> ServiceResult<bool> {
    let is_referee = multiplayer::is_referee(ctx, match_id, user_id).await?;
    Ok(is_referee)
}

pub async fn add_referee<C: Context>(
    ctx: &C,
    match_id: i64,
    user_id: i64,
    check_referee: Option<i64>,
) -> ServiceResult<()> {
    let mp_match = multiplayer::fetch_one(ctx, match_id)
        .await?
        .ok_or(AppError::MultiplayerNotFound)?;

    if let Some(check_referee) = check_referee
        && mp_match.host_user_id != check_referee
        && !is_referee(ctx, match_id, check_referee).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }
    multiplayer::add_referee(ctx, match_id, user_id).await?;
    Ok(())
}

pub async fn remove_referee<C: Context>(
    ctx: &C,
    match_id: i64,
    user_id: i64,
    check_referee: Option<i64>,
) -> ServiceResult<()> {
    let mp_match = multiplayer::fetch_one(ctx, match_id)
        .await?
        .ok_or(AppError::MultiplayerNotFound)?;

    if let Some(check_referee) = check_referee
        && mp_match.host_user_id != check_referee
        && !is_referee(ctx, match_id, check_referee).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }
    multiplayer::remove_referee(ctx, match_id, user_id).await?;
    Ok(())
}

pub async fn get_referees<C: Context>(ctx: &C, match_id: i64) -> ServiceResult<Vec<i64>> {
    let referees = multiplayer::get_referees(ctx, match_id).await?;
    Ok(referees)
}

pub fn start_timer<C: Context>(ctx: &C, match_id: i64, timer_type: TimerType, seconds: u64) {
    let state = AppState::from_ctx(ctx);
    tokio::spawn(async move {
        if let Err(e) = run_timer(state, match_id, timer_type, seconds).await {
            error!("Error running timer: {e:?}");
        }
    });
}

async fn run_timer<C: Context>(
    ctx: C,
    match_id: i64,
    timer_type: TimerType,
    seconds: u64,
) -> ServiceResult<()> {
    multiplayer::set_timer(&ctx, match_id, timer_type, seconds).await?;
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.tick().await;
    loop {
        interval.tick().await;
        // Timer was aborted externally
        if multiplayer::get_timer(&ctx, match_id, timer_type)
            .await?
            .is_none()
        {
            break;
        }

        let remaining_seconds = multiplayer::decrease_timer(&ctx, match_id, timer_type).await?;
        if remaining_seconds <= 0 {
            multiplayer::abort_timer(&ctx, match_id, timer_type).await?;
            match timer_type {
                TimerType::Regular => send_timer_ended_message(&ctx, match_id).await?,
                TimerType::MatchStart => start_game(&ctx, match_id, None).await?,
            }
            break;
        }
    }

    Ok(())
}

async fn send_timer_ended_message<C: Context>(ctx: &C, match_id: i64) -> ServiceResult<()> {
    let mp_match = fetch_one(ctx, match_id).await?;

    let bot_message = IrcMessage {
        sender_id: bot::BOT_ID as _,
        sender: bot::BOT_NAME,
        text: "Timer has ended",
        recipient: "#multiplayer",
    };
    streams::broadcast_message(
        ctx,
        StreamName::Multiplayer(mp_match.match_id),
        ChatMessage(&bot_message),
        None,
        None,
    )
    .await?;

    Ok(())
}

pub async fn abort_timer<C: Context>(
    ctx: &C,
    match_id: i64,
    timer_type: TimerType,
) -> ServiceResult<()> {
    multiplayer::abort_timer(ctx, match_id, timer_type).await?;
    Ok(())
}

pub async fn lock_match<C: Context>(ctx: &C, match_id: i64) -> ServiceResult<()> {
    // Lock all empty slots
    let mp_match = fetch_one(ctx, match_id).await?;
    let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    for slot in &mut slots {
        if slot.user.is_none() {
            slot.status = SlotStatus::Locked.bits();
        }
    }
    multiplayer::update_all_slots(ctx, match_id, slots).await?;
    broadcast_update(ctx, &mp_match, MultiplayerMatchSlot::from(slots)).await?;
    Ok(())
}

pub async fn unlock_match<C: Context>(ctx: &C, match_id: i64) -> ServiceResult<()> {
    let mp_match = fetch_one(ctx, match_id).await?;
    let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    for slot in &mut slots {
        if slot.status == SlotStatus::Locked.bits() {
            slot.status = SlotStatus::Empty.bits();
        }
    }
    multiplayer::update_all_slots(ctx, match_id, slots).await?;
    broadcast_update(ctx, &mp_match, MultiplayerMatchSlot::from(slots)).await?;
    Ok(())
}

pub async fn resize_match<C: Context>(
    ctx: &C,
    match_id: i64,
    new_size: usize,
) -> ServiceResult<()> {
    let mp_match = fetch_one(ctx, match_id).await?;
    let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;

    // Create a copy of all player slots
    let player_slots: Vec<_> = slots
        .iter()
        .filter(|slot| slot.user.is_some())
        .cloned()
        .collect();
    // Clear all slots and reset to empty
    for (i, slot) in slots.iter_mut().enumerate() {
        slot.clear();
        if i >= new_size {
            slot.status = SlotStatus::Locked.bits();
        }
    }

    // Place players at the beginning
    for (i, player_slot) in player_slots.iter().enumerate() {
        if i < new_size {
            slots[i] = player_slot.clone();
        }
    }

    // Update all slots
    multiplayer::update_all_slots(ctx, match_id, slots).await?;
    broadcast_update(ctx, &mp_match, MultiplayerMatchSlot::from(slots)).await?;
    Ok(())
}

pub async fn abort<C: Context>(ctx: &C, match_id: i64) -> ServiceResult<()> {
    let mut mp_match = multiplayer::fetch_one(ctx, match_id)
        .await?
        .ok_or(AppError::MultiplayerNotFound)?;

    if !mp_match.in_progress {
        return Ok(());
    }

    // Set match as not in progress
    mp_match.in_progress = false;

    // Reset all player slots to not ready
    let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    for slot in &mut slots {
        if slot.user.is_some() && slot.status == SlotStatus::Playing.bits() {
            slot.status = SlotStatus::NotReady.bits();
            slot.loaded = false;
            slot.skipped = false;
            slot.failed = false;
            slot.completed = false;
        }
    }

    // Of course just MatchAborted isn't enough...
    let match_aborted = concat_messages!(MatchAllPlayersLoaded, MatchAborted,);
    streams::broadcast_data(
        ctx,
        StreamName::Multiplaying(match_id),
        &match_aborted,
        None,
        None,
    )
    .await?;

    // Update match and slots
    let mp_match = multiplayer::update(ctx, mp_match, false).await?;
    multiplayer::update_all_slots(ctx, match_id, slots).await?;

    // Broadcast the updated match state
    let mp_match = MultiplayerMatch::try_from(mp_match)?;
    let slots = MultiplayerMatchSlot::from(slots);
    broadcast_update(ctx, &mp_match, slots).await?;

    // Save the match game end to match history
    match_games::game_ended(ctx, match_id).await?;

    Ok(())
}
// utility

async fn broadcast_update<C: Context>(
    ctx: &C,
    mp_match: &MultiplayerMatch,
    slots: MultiplayerMatchSlots,
) -> ServiceResult<()> {
    let bancho_match = mp_match.as_bancho(slots);
    let match_update = MatchUpdate(&bancho_match).as_message().serialize();
    streams::broadcast_data(ctx, StreamName::Lobby, &match_update, None, None).await?;
    streams::broadcast_data(
        ctx,
        StreamName::Multiplayer(mp_match.match_id),
        &match_update,
        None,
        None,
    )
    .await?;
    Ok(())
}

async fn update_match_members_presences<C: Context>(
    ctx: &C,
    user_ids: impl Iterator<Item = i64>,
    new_mode: Gamemode,
) -> ServiceResult<()> {
    // Update stats for all players in the match
    for user_id in user_ids {
        let mut presence = presences::fetch_one(ctx, user_id).await?;
        presence.action.mode = new_mode;

        // Fetch updated stats for the new mode
        let stats = stats::fetch_one(ctx, user_id, new_mode).await?;
        let global_rank = stats::fetch_global_rank(ctx, user_id, new_mode).await?;
        presence.stats = PresenceStats::from(stats, global_rank);
        let presence = presences::update(ctx, presence).await?;
        let user_panel = presence.user_panel();
        streams::broadcast_data(ctx, StreamName::Main, &user_panel, None, None).await?;
    }
    Ok(())
}

async fn change_playing_state<C: Context, F: Fn(&mut SlotEntity) -> &mut bool>(
    ctx: &C,
    match_id: i64,
    slot_session_id: Uuid,
    slot_map: F,
) -> ServiceResult<(bool, usize)> {
    let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    let mut all = true;
    let mut player_slot_id = None;
    slots
        .iter_mut()
        .filter(|slot| slot.user.is_some()) // only check slots with a user
        .enumerate()
        .for_each(|(id, slot)| {
            let slot_user = slot.user.unwrap();
            let value_binding = slot_map(slot);
            if slot_user.session_id == slot_session_id {
                *value_binding = true;
                slot.loaded = true;
                player_slot_id = Some(id);
            } else if !(*value_binding) {
                all = false;
            }
        });

    if player_slot_id.is_none() {
        return Err(AppError::MultiplayerUserNotInMatch);
    }
    let player_slot_id = player_slot_id.unwrap();
    let player_slot = slots[player_slot_id];
    multiplayer::update_slot(ctx, match_id, player_slot_id, player_slot).await?;
    Ok((all, player_slot_id))
}
