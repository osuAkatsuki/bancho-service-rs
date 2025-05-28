use crate::api::RequestContext;
use crate::common::error::AppError;
use crate::entities::bot;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::{sessions, spectators};
use bancho_protocol::concat_messages;
use bancho_protocol::messages::Message;
use bancho_protocol::messages::client::StartSpectating;
use bancho_protocol::messages::server::{
    Alert, FailedSpectating, FellowSpectatorJoined, SpectatorFrames,
};
use bancho_protocol::serde::osu_types::PrefixedVec;
use bancho_protocol::structures::{ReplayAction, ReplayFrameBundle, ScoreFrame};

pub async fn handle(ctx: &RequestContext, session: &Session, args: StartSpectating) -> EventResult {
    if args.target_id == (bot::BOT_ID as i32) {
        let alert = concat_messages!(
            // sending a SpectatorFrames message redirecting
            // the user to themselves; stops them from spectating client-side
            SpectatorFrames {
                frames: &ReplayFrameBundle {
                    extra: session.user_id as _,
                    frames: PrefixedVec::from(vec![]),
                    action: ReplayAction::WatchingOther,
                    score_frame: ScoreFrame::default(),
                    sequence: 0,
                }
            },
            Alert {
                message: "You can't spectate the bot.",
            }
        );
        return Ok(Some(alert));
    }

    let host_session = match sessions::fetch_one_by_user_id(ctx, args.target_id as _).await {
        Ok(session) if !session.is_publicly_visible() => {
            return Ok(Some(Message::serialize(FailedSpectating { user_id: 0 })));
        }
        Err(AppError::SessionsNotFound) => {
            return Ok(Some(Message::serialize(FailedSpectating { user_id: 0 })));
        }
        Err(e) => return Err(e),
        Ok(session) => session,
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
