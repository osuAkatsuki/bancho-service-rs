use crate::api::RequestContext;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::sessions;
use bancho_protocol::messages::client::SetAwayMessage;

pub async fn handle(
    ctx: &RequestContext,
    session: &mut Session,
    args: SetAwayMessage<'_>,
) -> EventResult {
    let away_message = match args.message.text.is_empty() {
        true => None,
        false => Some(args.message.text.to_owned()),
    };

    session.away_message = away_message;
    sessions::update(ctx, session.clone()).await?;

    Ok(None)
}
