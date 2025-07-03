use crate::api::RequestContext;
use crate::common::error::AppError;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::channels;
use bancho_protocol::messages::client::LeaveChannel;

pub async fn handle(
    ctx: &RequestContext,
    session: &Session,
    args: LeaveChannel<'_>,
) -> EventResult {
    match args.name {
        "#highlight" | "#userlog" => Ok(None),
        channel_name if !channel_name.starts_with('#') => Ok(None),
        channel_name => match channels::get_channel_name(ctx, session, channel_name).await {
            Ok(name) => {
                channels::leave(ctx, session.session_id, name).await?;
                Ok(None)
            }
            Err(AppError::MultiplayerUserNotInMatch) => Ok(None),
            Err(e) => Err(e),
        },
    }
}
