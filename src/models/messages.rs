use crate::entities::channels::ChannelName;
use crate::entities::messages::Message as MessageEntity;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use chrono::{DateTime, Utc};

#[derive(Copy, Clone)]
pub enum Recipient<'a> {
    Channel(ChannelName<'a>),
    UserSession(&'a Session),
    OfflineUser(&'a str),
    Bot,
}

pub enum MessageSendResult {
    Ok,
    CommandExecuted,
    CommandResponse(String),
}

pub struct Message {
    pub message_id: u64,
    pub sender_id: i64,
    pub sender_name: String,
    pub recipient_id: Option<i64>,
    pub recipient_channel: Option<String>,
    pub content: String,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<MessageEntity> for Message {
    fn from(value: MessageEntity) -> Self {
        Self {
            message_id: value.id,
            sender_id: value.sender_id,
            sender_name: value.sender_name,
            recipient_id: value.recipient_id,
            recipient_channel: value.recipient_channel,
            content: value.content,
            read_at: value.read_at,
            created_at: value.created_at,
            deleted_at: value.deleted_at,
        }
    }
}

impl<'a> Recipient<'a> {
    pub fn can_process_commands(&self) -> bool {
        match self {
            Recipient::Bot | Recipient::Channel(_) => true,
            _ => false,
        }
    }

    pub fn get_message_stream(self) -> Option<StreamName<'a>> {
        match self {
            Recipient::Channel(channel_name) => Some(channel_name.get_message_stream()),
            Recipient::UserSession(session) => Some(StreamName::User(session.session_id)),
            Recipient::Bot | Recipient::OfflineUser(_) => None,
        }
    }
}
