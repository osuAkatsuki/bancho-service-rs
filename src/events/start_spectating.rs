use crate::api::RequestContext;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::{sessions, spectators};
use bancho_protocol::messages::Message;
use bancho_protocol::messages::client::StartSpectating;
use bancho_protocol::messages::server::{FailedSpectating, FellowSpectatorJoined};

pub async fn handle(ctx: &RequestContext, session: &Session, args: StartSpectating) -> EventResult {
    let host_session = match sessions::fetch_one_by_user_id(ctx, args.target_id as _).await {
        Ok(session) => session,
        Err(AppError::SessionsNotFound) => {
            return Ok(Some(Message::serialize(FailedSpectating { user_id: 0 })));
        }
        Err(e) => return Err(e),
    };

    let spectator_ids = spectators::join(ctx, session, host_session).await?;
    let fellow_spectators = spectator_ids
        .into_iter()
        .map(|user_id| {
            Message::serialize(FellowSpectatorJoined {
                user_id: user_id as _,
            })
        })
        .flatten()
        .collect();
    Ok(Some(fellow_spectators))
}
