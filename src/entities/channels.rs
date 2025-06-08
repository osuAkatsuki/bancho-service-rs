use crate::common::error::ServiceResult;
use crate::repositories::streams::StreamName;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Copy, Clone)]
pub enum ChannelName<'a> {
    Spectator(Uuid),
    Multiplayer(i64),
    Chat(&'a str),
}

#[derive(sqlx::FromRow)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub public_read: bool,
    pub public_write: bool,
    pub status: bool,
}

impl<'a> ChannelName<'a> {
    pub fn from_key(key: &'a str) -> ServiceResult<Self> {
        match key.strip_prefix("#spectator_") {
            Some(host_session_id) => {
                let host_session_id = Uuid::from_str(host_session_id)?;
                Ok(ChannelName::Spectator(host_session_id))
            }
            None => match key.strip_prefix("#multiplayer_") {
                Some(match_id_str) => {
                    let match_id = i64::from_str(match_id_str)?;
                    Ok(ChannelName::Multiplayer(match_id))
                }
                None => Ok(ChannelName::Chat(key)),
            },
        }
    }

    pub fn to_bancho(&self) -> &str {
        match self {
            ChannelName::Chat(channel_name) => channel_name,
            ChannelName::Spectator(_) => "#spectator",
            ChannelName::Multiplayer(_) => "#multiplayer",
        }
    }

    pub fn get_message_stream(self) -> StreamName<'a> {
        StreamName::Channel(self)
    }

    pub fn get_update_stream(self) -> StreamName<'a> {
        match self {
            ChannelName::Spectator(host_session_id) => StreamName::Spectator(host_session_id),
            ChannelName::Multiplayer(match_id) => StreamName::Multiplayer(match_id),
            ChannelName::Chat(channel_name) => match channel_name {
                "#plus" | "#supporter" | "#premium" => StreamName::Donator,
                "#staff" => StreamName::Staff,
                "#devlog" => StreamName::Dev,
                _ => StreamName::Main,
            },
        }
    }
}

impl Display for ChannelName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelName::Spectator(uuid) => write!(f, "#spectator_{}", uuid),
            ChannelName::Multiplayer(match_id) => write!(f, "#multiplayer_{}", match_id),
            ChannelName::Chat(name) => Display::fmt(name, f),
        }
    }
}
