use crate::common::context::Context;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::{multiplayer, sessions};
use bancho_protocol::messages::client::MatchInvite;

pub async fn handle<C: Context>(ctx: &C, session: &Session, args: MatchInvite) -> EventResult {
    let targets = sessions::fetch_by_user_id(ctx, args.user_id as _).await?;
    multiplayer::invite(ctx, session, targets).await?;
    Ok(None)
}
