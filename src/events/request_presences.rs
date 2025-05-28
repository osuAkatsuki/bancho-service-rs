use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::presences;
use bancho_protocol::messages::Message;
use bancho_protocol::messages::client::RequestPresences;
use bancho_protocol::messages::server::UserLogout;
use tracing::info;

pub async fn handle(
    ctx: &RequestContext,
    session: &Session,
    args: RequestPresences,
) -> EventResult {
    info!(
        user_id = session.user_id,
        "User requested presences: {:?}", args.user_ids.0
    );
    let presences = presences::fetch_multiple(ctx, &args.user_ids.0).await?;
    let response = presences
        .into_iter()
        .map(|(user_id, p)| match p {
            None => Message::serialize(UserLogout::new(user_id)),
            Some(presence) if !presence.is_publicly_visible() => {
                Message::serialize(UserLogout::new(user_id))
            }
            Some(presence) => presence.user_panel(),
        })
        .flatten()
        .collect();
    Ok(Some(response))
}
