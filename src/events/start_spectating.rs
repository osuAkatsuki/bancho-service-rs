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

    if !session.is_publicly_visible() {
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
                message: "You are not allowed to spectate.",
            }
        );
        return Ok(Some(alert));
    }

    let spec_channel_notify = Message::serialize(ChannelJoinSuccess { name: "#spectator" });
    match spectators::join(ctx, session, args.target_id as _).await {
        Ok(spectators) => {
            match spectators.len() == 1 {
                // We are the only spectator
                true => Ok(Some(spec_channel_notify)),
                false => {
                    let mut response = spectators
                        .into_iter()
                        .filter(|spectator| session.session_id.ne(&spectator.session_id))
                        .flat_map(|spectator| {
                            Message::serialize(FellowSpectatorJoined {
                                user_id: spectator.user_id as _,
                            })
                        })
                        .collect::<Vec<_>>();
                    response.extend(spec_channel_notify);
                    Ok(Some(response))
                }
            }
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
