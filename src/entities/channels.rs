use crate::repositories::streams::StreamName;
use std::fmt::{Display, Formatter};
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
