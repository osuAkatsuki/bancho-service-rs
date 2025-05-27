use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::streams;
use bancho_protocol::messages::client::SpectateFrames;
use bancho_protocol::messages::server::SpectatorFrames;

pub async fn handle(ctx: &RequestContext, session: &Session, args: SpectateFrames) -> EventResult {
    let stream_name = StreamName::Spectator(session.session_id);
    if streams::is_joined(ctx, session.session_id, stream_name).await? {
        let excluded_session_ids = Some(vec![session.session_id]);
        streams::broadcast_message(
            ctx,
            stream_name,
            SpectatorFrames {
                frames: &args.frames,
            },
            excluded_session_ids,
            None,
        )
        .await?;
    }
    Ok(None)
}
