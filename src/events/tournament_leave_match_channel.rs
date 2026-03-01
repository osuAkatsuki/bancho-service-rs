use crate::api::RequestContext;
use crate::models::multiplayer::MultiplayerMatch;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{multiplayer, streams};

pub async fn handle(
    ctx: &RequestContext,
    session: &Session,
    match_id: i32,
) -> super::EventResult {
    let mp_match = multiplayer::fetch_all(ctx)
        .await?
        .into_iter()
        .find(|m: &MultiplayerMatch| (m.match_id & 0xFFFF) as i32 == match_id);

    match mp_match {
        Some(mp_match) => {
            tracing::info!(
                session_id = ?session.session_id,
                user_id = session.user_id,
                match_id = mp_match.match_id,
                "tournament client leaving match channel"
            );
            streams::leave(ctx, session.session_id, StreamName::Multiplayer(mp_match.match_id)).await?;
        }
        None => {
            tracing::debug!(
                session_id = ?session.session_id,
                match_id,
                "tournament client leaving match channel but match no longer exists, ignoring"
            );
        }
    }

    Ok(None)
}