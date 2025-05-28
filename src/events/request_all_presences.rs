use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::presences;
use bancho_protocol::messages::Message;
use bancho_protocol::messages::client::RequestAllPresences;
use bancho_protocol::messages::server::UserLogout;

pub async fn handle(
    ctx: &RequestContext,
    _session: &Session,
    _args: RequestAllPresences,
) -> EventResult {
    let presences = presences::fetch_all(ctx).await?;
    let response = presences
        .into_iter()
        .filter_map(|p| {
            if p.is_publicly_visible() {
                Some(p.user_panel())
            } else {
                Some(Message::serialize(UserLogout::new(p.user_id as _)))
            }
        })
        .flatten()
        .collect();
    Ok(Some(response))
}
