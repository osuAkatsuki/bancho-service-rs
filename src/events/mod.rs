pub mod change_action;
pub mod channel_join;
pub mod channel_leave;
pub mod login;
pub mod logout;
pub mod request_presences;
pub mod spectate_frames;
pub mod start_spectating;
pub mod stop_spectating;
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
                Err(AppError::SessionsNotFound | AppError::SessionsNeedsMigration) => {
                    BanchoResponse::error_raw(
                        None,
                        Message::serialize(Restart {
                            milliseconds: RECONNECT_DELAY,
                        }),
                    )
                }
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

macro_rules! event_handler {
    ($h:ident($ctx:expr, $session:expr)) => {
        $h::handle($ctx, $session).await
    };
    ($h:ident($ctx:expr, $session:expr, $event:expr)) => {
        $h::handle($ctx, $session, BinaryDeserialize::deserialize($event.data)?).await
    };
}

pub async fn handle_event(
    ctx: &RequestContext,
    session: &Session,
    event: Event<'_>,
) -> EventResult {
    match event.event_type {
        MessageType::Ping => Ok(None),
        MessageType::Logout => event_handler!(logout(ctx, session)),
        MessageType::ChangeAction => event_handler!(change_action(ctx, session, event)),
        MessageType::JoinChannel => event_handler!(channel_join(ctx, session, event)),
        MessageType::LeaveChannel => event_handler!(channel_leave(ctx, session, event)),
        MessageType::UserStatsRequest => event_handler!(user_stats_request(ctx, session, event)),
        MessageType::RequestPresences => event_handler!(request_presences(ctx, session, event)),
        MessageType::StartSpectating => event_handler!(start_spectating(ctx, session, event)),
        MessageType::StopSpectating => event_handler!(stop_spectating(ctx, session)),
        MessageType::SpectateFrames => event_handler!(spectate_frames(ctx, session, event)),
        _ => {
            warn!("Unhandled event: {:?}", event.event_type);
            Ok(None)
        }
    }
}

pub async fn handle_events(
    ctx: &RequestContext,
    session: Session,
    request_data: Bytes,
) -> BanchoResponse {
    // If the session is used from a different IP
    // than the IP that created it, we will log a warning
    if session.create_ip_address != ctx.request_ip.ip_addr {
        warn!(
            create_ip_address = session.create_ip_address.to_string(),
            request_ip_address = ctx.request_ip.ip_addr.to_string(),
            "Received events from an IP address different from the creation IP"
        );
    }

    let events = match Events::try_from(&request_data) {
        Ok(events) => events,
        Err(e) => return BanchoResponse::error(Some(session.session_id), e),
    };
    let mut response_data = vec![];
    for event in events.events {
        let event_type = event.event_type;
        match handle_event(ctx, &session, event).await {
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
