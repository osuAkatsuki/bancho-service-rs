use crate::api::RequestContext;
use crate::entities::channels::ChannelName;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{channels, multiplayer, streams};
use bancho_protocol::messages::Message;
use bancho_protocol::messages::server::MatchUpdate;

pub async fn handle(ctx: &RequestContext, session: &Session, match_id: i32) -> super::EventResult {
    let mp_match = multiplayer::fetch_one(ctx, match_id as i64).await?;

    tracing::info!(
        session_id = ?session.session_id,
        user_id = session.user_id,
        match_id = mp_match.match_id,
        "tournament client joining match channel"
    );

    streams::join(
        ctx,
        session.session_id,
        StreamName::Multiplayer(mp_match.match_id),
    )
    .await?;

    if session.privileges.is_tournament_staff() {
        channels::join(ctx, session, ChannelName::Multiplayer(mp_match.match_id)).await?;
    }

    let slots = multiplayer::fetch_all_slots(ctx, mp_match.match_id).await?;
    let bancho_match = mp_match.as_bancho(slots);
    let match_update = Message::serialize(MatchUpdate(&bancho_match));

    Ok(Some(match_update))
}
