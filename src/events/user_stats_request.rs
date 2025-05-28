use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::presences;
use bancho_protocol::messages::Message;
use bancho_protocol::messages::client::UserStatsRequest;
use bancho_protocol::messages::server::UserLogout;

pub async fn handle(
    ctx: &RequestContext,
    _session: &Session,
    args: UserStatsRequest,
) -> EventResult {
    let presences = presences::fetch_multiple(ctx, &args.user_ids.0).await?;
    let response = presences
        .into_iter()
        .map(|(user_id, p)| match p {
            None => Message::serialize(UserLogout::new(user_id)),
            Some(presence) if !presence.is_publicly_visible() => {
                Message::serialize(UserLogout::new(user_id))
            }
            Some(presence) if presence.stats.global_rank == 0 => vec![],
            Some(presence) => presence.user_panel(),
        })
        .flatten()
        .collect();
    Ok(Some(response))
}
