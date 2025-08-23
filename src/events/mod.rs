pub mod add_friend;
pub mod cant_spectate;
pub mod change_action;
pub mod channel_join;
pub mod channel_leave;
mod chat;
pub mod lobby_join;
pub mod lobby_leave;
pub mod login;
pub mod logout;
pub mod match_change_mods;
pub mod match_change_settings;
pub mod match_change_slot;
pub mod match_change_team;
pub mod match_create;
pub mod match_failed;
pub mod match_has_beatmap;
pub mod match_invite;
pub mod match_join;
pub mod match_leave;
pub mod match_loaded;
pub mod match_lock_slot;
pub mod match_no_beatmap;
pub mod match_not_ready;
pub mod match_player_complete;
pub mod match_ready;
pub mod match_request_skip;
pub mod match_start;
pub mod match_transfer_host;
pub mod match_update_score;
pub mod private_chat_message;
pub mod public_chat_message;
pub mod receive_updates;
pub mod remove_friend;
pub mod request_all_presences;
pub mod request_presences;
pub mod set_afk_message;
pub mod spectate_frames;
pub mod start_spectating;
pub mod stop_spectating;
pub mod toggle_private_dms;
pub mod update_stats_request;
pub mod user_stats_request;

use crate::api::RequestContext;
use crate::common::error::{AppError, ServiceResult};
use crate::models::bancho::{BanchoRequest, BanchoResponse};
use crate::models::sessions::Session;
use crate::usecases::{sessions, streams};
use axum::body::Bytes;
use bancho_protocol::messages::message::HEADER_SIZE;
use bancho_protocol::messages::server::{Alert, Restart};
use bancho_protocol::messages::{Message, MessageHeader, MessageType};
use bancho_protocol::serde::{BinaryDeserialize, BinaryReader};
use tracing::warn;

const RECONNECT_DELAY: u32 = 750;
pub async fn handle_request(ctx: &RequestContext, request: BanchoRequest) -> BanchoResponse {
    match request {
        BanchoRequest::Login(args) => login::handle(ctx, args).await,
        BanchoRequest::HandleEvents(session_id, request_data) => {
            match sessions::extend(ctx, session_id).await {
                Ok(session) => handle_events(ctx, session, request_data).await,
                Err(AppError::SessionsNotFound) => BanchoResponse::error_raw(
                    None,
                    Message::serialize(Restart {
                        milliseconds: RECONNECT_DELAY,
                    }),
                ),
                Err(e) => BanchoResponse::error(Some(session_id), e),
            }
        }
    }
}

pub struct Event<'a> {
    pub event_type: MessageType,
    pub data: &'a [u8],
}

pub struct Events<'a> {
    pub events: Vec<Event<'a>>,
}

pub type EventResult = ServiceResult<Option<Vec<u8>>>;

macro_rules! event_handlers {
    (
        $ctx:expr, $session:expr, $event:expr,
        [$($event_type:pat => $handler:expr),* $(,)?]
    ) => {
        match $event.event_type {
            $(
                $event_type =>
                    $handler($ctx, $session, BinaryDeserialize::deserialize($event.data)?).await,
            )*
            event_type => {
                warn!("Unhandled event: {event_type:?}");
                Ok(None)
            }
        }
    };
}

async fn ignore_event(_ctx: &RequestContext, _session: &Session, _args: ()) -> EventResult {
    Ok(None)
}

pub async fn handle_event(
    ctx: &RequestContext,
    session: &mut Session,
    event: Event<'_>,
) -> EventResult {
    event_handlers!(ctx, session, event, [
        // Ignored events
        MessageType::Ping
        | MessageType::CantSpectate
        | MessageType::ReceiveUpdates
        | MessageType::SetAwayMessage => ignore_event,
        // Miscellaneous events
        MessageType::Logout => logout::handle,
        MessageType::ChangeAction => change_action::handle,
        MessageType::RequestPresences => request_presences::handle,
        MessageType::RequestAllPresences => request_all_presences::handle,
        MessageType::ToggleBlockNonFriendDms => toggle_private_dms::handle,
        MessageType::UserStatsRequest => user_stats_request::handle,
        MessageType::UpdateStatsRequest => update_stats_request::handle,
        MessageType::AddFriend => add_friend::handle,
        MessageType::RemoveFriend => remove_friend::handle,

        // Chat events
        MessageType::JoinChannel => channel_join::handle,
        MessageType::LeaveChannel => channel_leave::handle,
        MessageType::PublicChatMessage => chat::public_chat_message,
        MessageType::PrivateChatMessage => chat::private_chat_message,

        // Spectator events
        MessageType::StartSpectating => start_spectating::handle,
        MessageType::StopSpectating => stop_spectating::handle,
        MessageType::SpectateFrames => spectate_frames::handle,

        // Multiplayer events
        MessageType::LeaveLobby => lobby_leave::handle,
        MessageType::JoinLobby => lobby_join::handle,
        MessageType::CreateMatch => match_create::handle,
        MessageType::JoinMatch => match_join::handle,
        MessageType::LeaveMatch => match_leave::handle,
        MessageType::MatchChangeSlot => match_change_slot::handle,
        MessageType::MatchReady => match_ready::handle,
        MessageType::MatchLock => match_lock_slot::handle,
        MessageType::MatchChangeSettings => match_change_settings::handle,
        MessageType::StartMatch => match_start::handle,
        MessageType::UpdateMatchScore => match_update_score::handle,
        MessageType::MatchPlayerComplete => match_player_complete::handle,
        MessageType::MatchChangeMods => match_change_mods::handle,
        MessageType::MatchLoadComplete => match_loaded::handle,
        MessageType::MatchNoBeatmap => match_no_beatmap::handle,
        MessageType::MatchNotReady => match_not_ready::handle,
        MessageType::MatchFailed => match_failed::handle,
        MessageType::MatchHasBeatmap => match_has_beatmap::handle,
        MessageType::MatchSkipRequest => match_request_skip::handle,
        MessageType::MatchChangeHost => match_transfer_host::handle,
        MessageType::MatchChangeTeam => match_change_team::handle,
        MessageType::MatchInvite => match_invite::handle,
        MessageType::MatchChangePassword => match_change_settings::handle,
    ])
}

pub async fn handle_events(
    ctx: &RequestContext,
    mut session: Session,
    request_data: Bytes,
) -> BanchoResponse {
    // If the session is used from a different IP
    // than the IP that created it, we will log a warning
    /*if session.create_ip_address != ctx.request_ip.ip_addr {
        warn!(
            create_ip_address = session.create_ip_address.to_string(),
            request_ip_address = ctx.request_ip.ip_addr.to_string(),
            "Received events from an IP address different from the creation IP"
        );
    }*/

    let events = match Events::try_from(&request_data) {
        Ok(events) => events,
        Err(e) => return BanchoResponse::error(Some(session.session_id), e),
    };
    let mut response_data = vec![];
    for event in events.events {
        let event_type = event.event_type;
        match handle_event(ctx, &mut session, event).await {
            Ok(None) => (),
            Ok(Some(data)) => response_data.extend_from_slice(&data),
            Err(e) => {
                let error_alert = Message::serialize(Alert {
                    message: e.message(),
                });
                response_data.extend_from_slice(&error_alert);
            }
        }

        if event_type == MessageType::Logout {
            return BanchoResponse::ok(session.session_id, response_data);
        }
    }

    let pending_data = streams::read_pending_data(ctx, &session)
        .await
        .unwrap_or_else(|e| {
            Message::serialize(Alert {
                message: e.message(),
            })
        });
    response_data.extend_from_slice(&pending_data);

    BanchoResponse::ok(session.session_id, response_data)
}

impl<'a> TryFrom<&'a Bytes> for Events<'a> {
    type Error = AppError;
    fn try_from(value: &'a Bytes) -> Result<Self, Self::Error> {
        let data = value.as_ref();
        let mut reader = BinaryReader::from(data);
        let mut events = vec![];
        while reader.can_read_n(HEADER_SIZE) {
            let header = MessageHeader::read_from(&mut reader)
                .map_err(|_| AppError::DecodingRequestFailed)?;
            let event_data = reader
                .next_range(header.args_len as _)
                .map_err(|_| AppError::DecodingRequestFailed)?;
            let event = Event {
                event_type: header.message_type,
                data: event_data,
            };
            events.push(event);
        }
        Ok(Self { events })
    }
}
