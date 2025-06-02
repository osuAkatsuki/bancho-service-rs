use crate::api::RequestContext;
use crate::common::error::AppError;
use crate::entities::bot;
use crate::events::EventResult;
use crate::models::sessions::Session;
use crate::usecases::spectators;
use bancho_protocol::concat_messages;
use bancho_protocol::messages::Message;
use bancho_protocol::messages::client::StartSpectating;
use bancho_protocol::messages::server::{
    Alert, ChannelJoinSuccess, FellowSpectatorJoined, SpectatorFrames, UserLogout,
};
use bancho_protocol::serde::osu_types::PrefixedVec;
use bancho_protocol::structures::{ReplayAction, ReplayFrameBundle, ScoreFrame};

pub async fn handle(ctx: &RequestContext, session: &Session, args: StartSpectating) -> EventResult {
    if args.target_id == (bot::BOT_ID as i32) {
        let alert = concat_messages!(
            // redirecting the user to themselves forces the client to stop spectating
            SpectatorFrames {
                frames: &ReplayFrameBundle {
                    action: ReplayAction::WatchingOther,
                    extra: session.user_id as _,
                    frames: PrefixedVec::from(vec![]),
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

    match spectators::join(ctx, session, args.target_id as _).await {
        Ok(spectator_ids) => {
            let mut fellow_spectators = spectator_ids
                .into_iter()
                .flat_map(|user_id| {
                    Message::serialize(FellowSpectatorJoined {
                        user_id: user_id as _,
                    })
                })
                .collect::<Vec<_>>();
            fellow_spectators.extend(Message::serialize(ChannelJoinSuccess {
                name: "#spectator",
            }));
            Ok(Some(fellow_spectators))
        }
        Err(e @ (AppError::InteractionBlocked | AppError::SessionsNotFound)) => {
            let notification = concat_messages!(
                UserLogout::new(args.target_id as _),
                Alert {
                    message: e.message()
                }
            );
            Ok(Some(notification))
        }
        Err(e) => Err(e),
    }
}
