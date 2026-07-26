use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::repositories::spectators;
use crate::repositories::streams::StreamName;
use crate::usecases::streams;
use bancho_protocol::messages::server::FailedSpectating;

pub async fn handle(ctx: &RequestContext, session: &Session, _args: ()) -> EventResult {
    let host_session_id = match spectators::fetch_spectating(ctx, session.session_id).await? {
        Some(id) => id,
        None => return Ok(None),
    };

    let host_stream_name = StreamName::User(host_session_id);
    streams::broadcast_message(
        ctx,
        host_stream_name,
        FailedSpectating {
            user_id: session.user_id as _,
        },
        None,
        None,
    )
    .await?;

    Ok(None)
}
