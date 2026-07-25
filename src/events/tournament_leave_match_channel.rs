use crate::api::RequestContext;
use crate::entities::channels::ChannelName;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{channels, multiplayer, streams};

pub async fn handle(ctx: &RequestContext, session: &Session, match_id: i32) -> super::EventResult {
    let mp_match = multiplayer::fetch_one(ctx, match_id as i64).await?;

    tracing::info!(
        session_id = ?session.session_id,
        user_id = session.user_id,
        match_id = mp_match.match_id,
        "tournament client leaving match channel"
    );

    streams::leave(
        ctx,
        session.session_id,
        StreamName::Multiplayer(mp_match.match_id),
    )
    .await?;
    channels::leave(
        ctx,
        session.session_id,
        ChannelName::Multiplayer(mp_match.match_id),
    )
    .await?;

    Ok(None)
}
