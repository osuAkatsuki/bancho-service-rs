use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{multiplayer, streams};
use bancho_protocol::messages::MessageArgs;
use bancho_protocol::messages::server::MatchUpdate;
use bancho_protocol::serde::BinarySerialize;

pub async fn handle(ctx: &RequestContext, session: &Session) -> EventResult {
    streams::join(ctx, session.session_id, StreamName::Lobby).await?;
    let matches = multiplayer::fetch_all_with_slots(ctx).await?;
    let response = matches
        .into_iter()
        .flat_map(|(mp_match, slots)| {
            MatchUpdate(&mp_match.as_bancho(slots))
                .as_message()
                .serialize()
        })
        .collect();
    Ok(Some(response))
}
