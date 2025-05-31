use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::entities::match_events::MatchEventType;
use crate::models::Gamemode;
use crate::models::multiplayer::{MultiplayerMatch, MultiplayerMatchSlot, MultiplayerMatchSlots};
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::repositories::{match_events, multiplayer};
use crate::usecases::streams;
use bancho_protocol::messages::server::{MatchCreated, MatchUpdate};

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
        leave(ctx, host_session, Some(match_id)).await?;
    }

    let (mp_match, slots) = multiplayer::create(
        ctx,
        host_session.session_id,
        host_session.user_id,
        name,
        password,
        beatmap_name,
        beatmap_md5,
        beatmap_id,
        mode as _,
        max_player_count,
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
    match_events::create(
        ctx,
        mp_match.match_id,
        MatchEventType::MatchCreated,
        Some(mp_match.host_user_id),
        None,
    )
    .await?;

    let match_notification = mp_match.to_bancho(slots);
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

pub async fn join<C: Context>(
    ctx: &C,
    session: &Session,
    match_id: i64,
    password: &str,
) -> ServiceResult<(MultiplayerMatch, MultiplayerMatchSlots)> {
    if let Some(match_id) = multiplayer::fetch_session_match_id(ctx, session.session_id).await? {
        leave(ctx, session, Some(match_id)).await?;
    }

    let mp_match = fetch_one(ctx, match_id).await?;
    if mp_match.password != password {
        return Err(AppError::MultiplayerInvalidPassword);
    }

    streams::leave(ctx, session.session_id, StreamName::Lobby).await?;
    let slots = match multiplayer::join(ctx, session.session_id, session.user_id, mp_match.match_id)
        .await?
    {
        Some(slots) => MultiplayerMatchSlot::from(slots),
        None => return Err(AppError::MultiplayerMatchFull),
    };

    match match_events::create(
        ctx,
        match_id,
        MatchEventType::MatchUserJoined,
        Some(session.user_id),
        None,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            let _ = unexpected::<(), _>(e);
        }
    }

    streams::join(
        ctx,
        session.session_id,
        StreamName::Multiplayer(mp_match.match_id),
    )
    .await?;

    let match_update = mp_match.to_bancho(slots);
    streams::broadcast_message(
        ctx,
        StreamName::Multiplayer(match_id),
        MatchUpdate(&match_update),
        None,
        None,
    )
    .await?;
    Ok((mp_match, slots))
}

pub async fn leave<C: Context>(
    ctx: &C,
    session: &Session,
    match_id: Option<i64>,
) -> ServiceResult<()> {
    let match_id = match match_id {
        Some(match_id) => match_id,
        None => match multiplayer::fetch_session_match_id(ctx, session.session_id).await? {
            Some(match_id) => match_id,
            None => return Ok(()),
        },
    };

    let mp_match = fetch_one(ctx, match_id).await?;
    let (user_count, slots) =
        match multiplayer::leave(ctx, session.session_id, session.user_id, mp_match.match_id)
            .await?
        {
            Some((user_count, slots)) => (user_count, MultiplayerMatchSlot::from(slots)),
            None => return Ok(()),
        };

    match match_events::create(
        ctx,
        match_id,
        MatchEventType::MatchUserLeft,
        Some(session.user_id),
        None,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            let _ = unexpected::<(), _>(e);
        }
    }

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

    if user_count == 0 {
        streams::clear_stream(ctx, StreamName::Multiplayer(mp_match.match_id)).await?;
        streams::clear_stream(ctx, StreamName::Multiplaying(mp_match.match_id)).await?;

        match match_events::create(ctx, match_id, MatchEventType::MatchDisbanded, None, None).await
        {
            Ok(_) => {}
            Err(e) => {
                let _ = unexpected::<(), _>(e);
            }
        }
    } else {
        let match_update = mp_match.to_bancho(slots);
        streams::broadcast_message(
            ctx,
            StreamName::Multiplayer(match_id),
            MatchUpdate(&match_update),
            None,
            None,
        )
        .await?;
    }
    Ok(())
}
