use crate::commands;
use crate::commands::{COMMAND_PREFIX, CommandResponse};
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::entities::bot;
use crate::entities::channels::ChannelName;
use crate::models::messages::{Message, MessageSendResult, Recipient};
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::messages;
use crate::usecases::{channels, relationships, streams, users};
use bancho_protocol::messages::MessageArgs;
use bancho_protocol::messages::server::{Alert, ChatMessage, TargetSilenced};
use bancho_protocol::serde::BinarySerialize;
use bancho_protocol::structures::IrcMessage;
use tracing::error;
use uuid::Uuid;

const CHAT_SPAM_RATE_INTERVAL: u64 = 10;
const CHAT_SPAM_RATE: i64 = 10;
const CHAT_TIMEOUT_SECONDS: i64 = 5 * 60;
const CHAT_TIMEOUT_REASON: &str = "Spamming (Auto-Silence)";

pub async fn fetch_unread_messages<C: Context>(
    ctx: &C,
    recipient_id: i64,
) -> ServiceResult<impl Iterator<Item = Message>> {
    match messages::fetch_unread_messages(ctx, recipient_id).await {
        Ok(messages) => Ok(messages.into_iter().map(Message::from)),
        Err(e) => unexpected(e),
    }
}

pub async fn mark_all_read<C: Context>(ctx: &C, recipient_id: i64) -> ServiceResult<()> {
    match messages::mark_all_read(ctx, recipient_id).await {
        Ok(()) => Ok(()),
        Err(e) => unexpected(e),
    }
}

pub async fn delete_recent<C: Context>(
    ctx: &C,
    sender_id: i64,
    delta_seconds: u64,
) -> ServiceResult<()> {
    match messages::delete_recent(ctx, sender_id, delta_seconds).await {
        Ok(()) => Ok(()),
        Err(e) => unexpected(e),
    }
}

struct RecipientInfo<'a> {
    recipient_channel: Option<ChannelName<'a>>,
    recipient_id: Option<i64>,
    mark_as_unread: bool,
    excluded_session_ids: Option<Vec<Uuid>>,
}

impl Default for RecipientInfo<'_> {
    fn default() -> Self {
        Self {
            recipient_channel: None,
            recipient_id: None,
            mark_as_unread: false,
            excluded_session_ids: None,
        }
    }
}

pub async fn send<C: Context>(
    ctx: &C,
    session: &Session,
    recipient: Recipient<'_>,
    args: &IrcMessage<'_>,
) -> ServiceResult<MessageSendResult> {
    if session.is_silenced() {
        return Ok(MessageSendResult::Ok);
    }

    if args.text.len() > 500 {
        return Err(AppError::MessagesTooLong);
    }

    let message_count =
        messages::message_count(ctx, session.user_id, CHAT_SPAM_RATE_INTERVAL).await?;
    if message_count >= CHAT_SPAM_RATE {
        users::silence_user(ctx, session, CHAT_TIMEOUT_REASON, CHAT_TIMEOUT_SECONDS).await?;
        return Err(AppError::MessagesUserAutoSilenced);
    }

    let recipient_info = get_recipient_info(ctx, session, recipient).await?;
    messages::send(
        ctx,
        session.user_id,
        recipient_info.recipient_channel,
        recipient_info.recipient_id,
        args.text,
        recipient_info.mark_as_unread,
    )
    .await?;

    let response = match commands::is_command_message(args.text) && recipient.can_process_commands()
    {
        true => commands::handle_command(ctx, session, args.text).await?,
        false => CommandResponse::default(),
    };
    let properties = response.properties;

    if let Some(stream_name) = recipient.get_message_stream() {
        if properties.forward_message {
            let msg = IrcMessage {
                sender: &session.username,
                text: args.text,
                recipient: args.recipient,
                sender_id: session.user_id as _,
            };
            streams::broadcast_message(
                ctx,
                stream_name,
                ChatMessage(&msg),
                recipient_info.excluded_session_ids,
                properties.read_privileges,
            )
            .await?;
        }
    }

    match response.answer {
        Some(answer) => match properties.forward_message {
            true => match recipient.get_message_stream() {
                Some(stream_name) => {
                    let msg = IrcMessage {
                        sender: bot::BOT_NAME,
                        text: &answer,
                        recipient: args.recipient,
                        sender_id: bot::BOT_ID as _,
                    };
                    streams::broadcast_message(
                        ctx,
                        stream_name,
                        ChatMessage(&msg),
                        None,
                        properties.read_privileges,
                    )
                    .await?;
                    Ok(MessageSendResult::Ok)
                }
                _ => Ok(MessageSendResult::CommandResponse(answer)),
            },
            false => Ok(MessageSendResult::CommandResponse(answer)),
        },
        None => Ok(MessageSendResult::Ok),
    }
}

pub async fn send_bancho<C: Context>(
    ctx: &C,
    session: &Session,
    recipient: Recipient<'_>,
    message: IrcMessage<'_>,
) -> ServiceResult<Option<Vec<u8>>> {
    match send(ctx, session, recipient, &message).await {
        Ok(result) => match recipient {
            Recipient::UserSession(recipient_session) if recipient_session.is_silenced() => {
                let response = TargetSilenced::new(&recipient_session.username);
                Ok(Some(response.as_message().serialize()))
            }
            Recipient::OfflineUser(username) => {
                let offline_msg = IrcMessage {
                    sender: username,
                    text: "\x01ACTION is offline right now. They will receive your message when they come back.",
                    recipient: &session.username,
                    sender_id: 0,
                };
                Ok(Some(ChatMessage(&offline_msg).as_message().serialize()))
            }
            _ => match result {
                MessageSendResult::Ok | MessageSendResult::CommandExecuted => Ok(None),
                MessageSendResult::CommandResponse(response) => {
                    let cmd_response = IrcMessage {
                        sender: bot::BOT_NAME,
                        text: &response,
                        recipient: &session.username,
                        sender_id: bot::BOT_ID as _,
                    };
                    Ok(Some(ChatMessage(&cmd_response).as_message().serialize()))
                }
            },
        },
        Err(
            e @ (AppError::InteractionBlocked
            | AppError::ChannelsUnauthorized
            | AppError::CommandsUnauthorized
            | AppError::MessagesTooLong
            | AppError::MessagesUserAutoSilenced
            | AppError::UsersNotFound),
        ) => {
            let alert = Alert {
                message: e.message(),
            };
            Ok(Some(alert.as_message().serialize()))
        }
        Err(AppError::CommandsInvalidSyntax(syntax, _, typed)) => {
            let recipient = match recipient {
                Recipient::Bot => session.username.as_str(),
                Recipient::Channel(ref channel_name) => channel_name.to_bancho(),
                _ => unreachable!(),
            };
            let cmd_name = match message.text.find(' ') {
                None => &message.text[1..],
                Some(idx) => &message.text[1..idx],
            };
            let text = format!(
                "Invalid Command Syntax! Correct Syntax: {COMMAND_PREFIX}{cmd_name} {syntax}\n{typed}",
            );
            let syntax_message = IrcMessage {
                recipient,
                text: text.as_str(),
                sender: bot::BOT_NAME,
                sender_id: bot::BOT_ID as _,
            };
            Ok(Some(ChatMessage(&syntax_message).as_message().serialize()))
        }
        Err(e) => Err(e),
    }
}

async fn get_recipient_info<'a, C: Context>(
    ctx: &C,
    sender: &Session,
    recipient: Recipient<'a>,
) -> ServiceResult<RecipientInfo<'a>> {
    match recipient {
        Recipient::Channel(channel_name) => {
            let channel = channels::fetch_one(ctx, channel_name).await?;
            if !channel.can_write(sender.privileges) {
                return Err(AppError::ChannelsUnauthorized);
            }

            Ok(RecipientInfo {
                recipient_channel: Some(channel_name),
                excluded_session_ids: Some(vec![sender.session_id]),
                ..Default::default()
            })
        }
        Recipient::UserSession(receiver_session) => {
            if !receiver_session.is_publicly_visible()
                && !sender.has_all_privileges(Privileges::AdminCaker)
            {
                return Err(AppError::InteractionBlocked);
            }
            if receiver_session.private_dms {
                match relationships::fetch_one(ctx, receiver_session.user_id, sender.user_id).await
                {
                    Err(AppError::RelationshipsNotFound) => {
                        return Err(AppError::InteractionBlocked);
                    }
                    Ok(_) => {}
                    Err(e) => error!("Error fetching relationship: {e:?}"),
                }
            }
            Ok(RecipientInfo {
                recipient_id: Some(receiver_session.user_id),
                ..Default::default()
            })
        }
        Recipient::OfflineUser(username) => {
            let user = users::fetch_one_by_username(ctx, username).await?;
            if !user.privileges.is_publicly_visible()
                && !sender.has_all_privileges(Privileges::AdminCaker)
            {
                return Err(AppError::InteractionBlocked);
            }

            Ok(RecipientInfo {
                recipient_id: Some(user.user_id),
                mark_as_unread: true,
                ..Default::default()
            })
        }
        Recipient::Bot => Ok(RecipientInfo {
            recipient_id: Some(bot::BOT_ID),
            ..Default::default()
        }),
    }
}
