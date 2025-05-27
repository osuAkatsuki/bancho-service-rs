use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::{presences, users};
use bancho_protocol::concat_messages;
use bancho_protocol::messages::Message;
use bancho_protocol::messages::client::RequestPresences;
use bancho_protocol::messages::server::{UserLogout, UserPresence};
use tracing::{error, info};

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
    let mut presences_data: Vec<u8> = vec![];
    for (user_id, presence) in presences {
        let response = match presence {
            None => Message::serialize(UserLogout::new(user_id as _)),
            Some(presence) => match users::fetch_one(ctx, presence.user_id).await {
                Ok(user) if user.privileges.is_publicly_visible() => concat_messages! {
                    UserPresence::new(
                        presence.user_id as _,
                        &user.username,
                        presence.location.utc_offset,
                        presence.location.country,
                        presence.action.mode.to_bancho(),
                        session.privileges.to_bancho(),
                        presence.location.latitude,
                        presence.location.longitude,
                    ),
                    presence.to_bancho(),
                },
                Ok(_) => Message::serialize(UserLogout::new(presence.user_id as _)),
                Err(e) => {
                    error!("Failed to fetch user: {:?}", e);
                    Message::serialize(UserLogout::new(presence.user_id as _))
                }
            },
        };
        presences_data.extend_from_slice(&response);
    }
    Ok(Some(presences_data))
}
