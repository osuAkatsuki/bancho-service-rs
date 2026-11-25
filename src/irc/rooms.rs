use super::protocol::{Client, match_id};
use super::{SERVER, service_error};
use crate::commands::CommandTarget;
use crate::common::error::{AppError, ServiceResult};
use crate::common::state::AppState;
use crate::common::website;
use crate::entities::bot;
use crate::entities::channels::ChannelName;
use crate::models::messages::Recipient;
use crate::models::multiplayer::MultiplayerMatch;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::streams::{self as stream_repository, StreamName};
use crate::usecases::{messages, multiplayer, streams};
use bancho_protocol::messages::message::HEADER_SIZE;
use bancho_protocol::messages::server::ChatMessage;
use bancho_protocol::messages::{MessageHeader, MessageType};
use bancho_protocol::serde::{BinaryDeserialize, BinaryReader};
use bancho_protocol::structures::{IrcMessage, Match, SlotStatus};
use std::collections::BTreeMap;
use tokio::io::{AsyncRead, AsyncWrite};

const MAX_ROOMS: usize = 8;

#[derive(Default)]
pub struct Rooms(BTreeMap<i64, RoomSubscription>);

struct RoomSubscription {
    chat_cursor: String,
    match_cursor: String,
    playing: bool,
    all_ready: bool,
}

async fn room_for_referee(
    state: &AppState,
    session: &Session,
    match_id: i64,
) -> ServiceResult<Option<MultiplayerMatch>> {
    let room = match multiplayer::fetch_one(state, match_id).await {
        Ok(room) => room,
        Err(AppError::MultiplayerNotFound) => return Ok(None),
        Err(error) => return Err(error),
    };
    if room.tournament_owner_id.is_none()
        || !multiplayer::can_manage(state, match_id, room.controller_user_id(), session.user_id)
            .await?
    {
        return Ok(None);
    }
    Ok(Some(room))
}

impl Rooms {
    pub async fn join<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send>(
        &mut self,
        state: &AppState,
        client: &mut Client<R, W>,
        session: &Session,
        nick: &str,
        channel: &str,
    ) -> anyhow::Result<()> {
        let Some(match_id) = match_id(channel) else {
            return client.numeric("403", nick, "* :Use #mp_<match_id>").await;
        };
        if self.0.contains_key(&match_id) {
            return Ok(());
        }
        if self.0.len() >= MAX_ROOMS {
            return client
                .numeric("405", nick, &format!("{channel} :Too many channels"))
                .await;
        }
        let chat_cursor = stream_repository::get_latest_message_id(
            state,
            ChannelName::Multiplayer(match_id).get_message_stream(),
        )
        .await?;
        let match_cursor =
            stream_repository::get_latest_message_id(state, StreamName::Multiplayer(match_id))
                .await?;
        let Some(room) = room_for_referee(state, session, match_id)
            .await
            .map_err(service_error)?
        else {
            return client
                .numeric(
                    "473",
                    nick,
                    &format!("{channel} :Tournament room access denied"),
                )
                .await;
        };
        self.0.insert(
            match_id,
            RoomSubscription {
                chat_cursor,
                match_cursor,
                playing: room.in_progress,
                all_ready: false,
            },
        );

        client
            .send(&format!(":{nick}!{nick}@{SERVER} JOIN {channel}"))
            .await?;
        client
            .numeric(
                "353",
                nick,
                &format!("= {channel} :{} {nick}", bot::BOT_NAME),
            )
            .await?;
        client
            .numeric("366", nick, &format!("{channel} :End of NAMES list"))
            .await
    }

    pub async fn part<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send>(
        &mut self,
        client: &mut Client<R, W>,
        nick: &str,
        channel: &str,
    ) -> anyhow::Result<()> {
        if let Some(match_id) = match_id(channel)
            && self.0.remove(&match_id).is_some()
        {
            return client
                .send(&format!(":{nick}!{nick}@{SERVER} PART {channel}"))
                .await;
        }
        client
            .numeric("442", nick, "* :You are not on that channel")
            .await
    }

    pub async fn message<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send>(
        &self,
        state: &AppState,
        client: &mut Client<R, W>,
        session: &mut Session,
        nick: &str,
        channel: &str,
        text: &str,
    ) -> anyhow::Result<()> {
        let Some(match_id) = match_id(channel) else {
            return client
                .numeric("401", nick, "* :Use Aika or a tournament room")
                .await;
        };
        if !self.0.contains_key(&match_id)
            || room_for_referee(state, session, match_id)
                .await
                .map_err(service_error)?
                .is_none()
        {
            return client
                .numeric("404", nick, &format!("{channel} :Cannot send to channel"))
                .await;
        }
        let text = text.trim();
        if text.is_empty() {
            return Ok(());
        }
        let command = text.split_whitespace().next().unwrap_or("");
        if command.starts_with('!') && !matches!(command, "!mp" | "!roll") {
            return client
                .message(
                    bot::BOT_NAME,
                    nick,
                    "IRC supports !mp and !roll in tournament rooms.",
                )
                .await;
        }

        let room_channel = ChannelName::Multiplayer(match_id);
        let recipient = Recipient::Channel(room_channel);
        let result = messages::send(
            state,
            session,
            &recipient,
            text,
            CommandTarget::IrcRoom(match_id),
        )
        .await;
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                return client
                    .message(bot::BOT_NAME, channel, error.message())
                    .await;
            }
        };

        let stream = room_channel.get_message_stream();
        if let Some(response) = result.response {
            let subcommand = text.split_whitespace().nth(1);
            let forward = response.properties.forward_message
                && !matches!(subcommand, Some("settings" | "results" | "close"));
            if forward {
                send_to_game(
                    state,
                    stream,
                    &session.username,
                    session.user_id,
                    &result.message.content,
                    Some(session.session_id),
                    response.properties.read_privileges,
                )
                .await?;
            }
            if let Some(answer) = response.answer {
                client.message(bot::BOT_NAME, channel, &answer).await?;
                if forward {
                    send_to_game(
                        state,
                        stream,
                        bot::BOT_NAME,
                        bot::BOT_ID,
                        &answer,
                        Some(session.session_id),
                        response.properties.read_privileges,
                    )
                    .await?;
                }
            }
        } else {
            send_to_game(
                state,
                stream,
                &session.username,
                session.user_id,
                &result.message.content,
                Some(session.session_id),
                None,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn relay<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send>(
        &mut self,
        state: &AppState,
        client: &mut Client<R, W>,
        session: &Session,
        nick: &str,
    ) -> anyhow::Result<()> {
        let offsets: Vec<_> = self
            .0
            .iter()
            .flat_map(|(&id, room)| {
                [
                    (
                        ChannelName::Multiplayer(id).get_message_stream(),
                        room.chat_cursor.as_str(),
                    ),
                    (StreamName::Multiplayer(id), room.match_cursor.as_str()),
                ]
            })
            .collect();

        for reply in stream_repository::read_streams(state, &offsets).await? {
            let (id, chat) =
                match StreamName::from_key(&reply.stream_name).map_err(service_error)? {
                    StreamName::Channel(ChannelName::Multiplayer(id)) => (id, true),
                    StreamName::Multiplayer(id) => (id, false),
                    _ => continue,
                };
            let channel = format!("#mp_{id}");
            if room_for_referee(state, session, id)
                .await
                .map_err(service_error)?
                .is_none()
            {
                self.part(client, nick, &channel).await?;
                continue;
            }
            let Some(room) = self.0.get_mut(&id) else {
                continue;
            };
            let cursor = if chat {
                &mut room.chat_cursor
            } else {
                &mut room.match_cursor
            };
            if let Some(last) = reply.messages.last() {
                cursor.clone_from(&last.message_id);
            }
            for message in reply.messages {
                if message
                    .info
                    .excluded_session_ids
                    .is_some_and(|ids| ids.contains(&session.session_id))
                    || message.info.read_privileges.is_some_and(|privileges| {
                        !session.has_all_privileges(Privileges::from_bits_retain(privileges))
                    })
                {
                    continue;
                }
                relay_packets(client, &channel, id, room, &message.data).await?;
            }
        }

        // A closed room's streams are deleted, so Redis cannot notify its IRC subscribers.
        // Recheck joined rooms on the poll even when nothing was written to them.
        let ids: Vec<_> = self.0.keys().copied().collect();
        for id in ids {
            if room_for_referee(state, session, id)
                .await
                .map_err(service_error)?
                .is_none()
            {
                self.part(client, nick, &format!("#mp_{id}")).await?;
            }
        }
        Ok(())
    }
}

async fn send_to_game(
    state: &AppState,
    stream: StreamName<'_>,
    sender: &str,
    user_id: i64,
    text: &str,
    exclude: Option<uuid::Uuid>,
    read_privileges: Option<Privileges>,
) -> anyhow::Result<()> {
    let message = IrcMessage {
        sender,
        text,
        recipient: "#multiplayer",
        sender_id: user_id as i32,
    };
    streams::broadcast_message(
        state,
        stream,
        ChatMessage(&message),
        exclude.map(|id| vec![id]),
        read_privileges,
    )
    .await
    .map_err(service_error)
}

async fn relay_packets<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send>(
    client: &mut Client<R, W>,
    channel: &str,
    id: i64,
    room: &mut RoomSubscription,
    data: &[u8],
) -> anyhow::Result<()> {
    let mut reader = BinaryReader::from(data);
    while reader.can_read_n(HEADER_SIZE) {
        let header = MessageHeader::read_from(&mut reader)?;
        let packet = reader.next_range(header.args_len as usize)?;
        match header.message_type {
            MessageType::ChatMessage => {
                let message = IrcMessage::deserialize(packet)?;
                let sender = message.sender.replace(' ', "_");
                if super::valid_nick(&sender) {
                    client.message(&sender, channel, message.text).await?;
                }
            }
            MessageType::MatchStart => {
                if !room.playing {
                    client
                        .message(bot::BOT_NAME, channel, "The match has started.")
                        .await?;
                }
                room.playing = true;
                room.all_ready = false;
            }
            MessageType::MatchUpdate => {
                let lobby = Match::deserialize(packet)?;
                if room.playing && !lobby.in_progress {
                    client
                        .message(
                            bot::BOT_NAME,
                            channel,
                            &format!(
                                "The round ended. Use !mp results or view {}.",
                                website::get_match_history_link(id)
                            ),
                        )
                        .await?;
                }
                room.playing = lobby.in_progress;
                let all_ready = all_players_ready(&lobby);
                if all_ready && !room.all_ready {
                    client
                        .message(bot::BOT_NAME, channel, "All players are ready.")
                        .await?;
                }
                room.all_ready = all_ready;
            }
            _ => {}
        }
    }
    Ok(())
}

fn all_players_ready(lobby: &Match<'_>) -> bool {
    let mut has_players = false;
    for slot in lobby.slots {
        if slot.user_id != 0 {
            has_players = true;
            if slot.status != SlotStatus::Ready {
                return false;
            }
        }
    }
    has_players
}
