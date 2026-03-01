use crate::api::RequestContext;
use crate::models::sessions::Session;
use crate::usecases::multiplayer;
use bancho_protocol::messages::Message;
use bancho_protocol::messages::server::MatchUpdate;

pub async fn handle(ctx: &RequestContext, session: &Session, match_id: i32) -> super::EventResult {
    let mp_match = multiplayer::fetch_one(ctx, match_id as _).await?;

    tracing::debug!(
        session_id = ?session.session_id,
        user_id = session.user_id,
        match_id = mp_match.match_id,
        "tournament client requesting match info"
    );

    let slots = multiplayer::fetch_all_slots(ctx, mp_match.match_id).await?;
    let bancho_match = mp_match.as_bancho(slots);
    let match_update = Message::serialize(MatchUpdate(&bancho_match));

    Ok(Some(match_update))
}
