use crate::commands;
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::entities::bot;
use crate::models::messages::{Message, Recipient};
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::messages;
use crate::usecases::{channels, streams, users};
use bancho_protocol::concat_messages;
use bancho_protocol::messages::MessageArgs;
use bancho_protocol::messages::server::{Alert, ChatMessage, TargetSilenced, UserSilenced};
use bancho_protocol::serde::BinarySerialize;
use bancho_protocol::structures::IrcMessage;

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

pub async fn send<C: Context>(
    ctx: &C,
    session: &Session,
    recipient: Recipient<'_>,
    args: IrcMessage<'_>,
) -> ServiceResult<()> {
    if session.is_silenced() {
        return Ok(());
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

    let mut recipient_channel = None;
    let mut recipient_id = None;
    let mut mark_as_unread = false;
    let mut excluded_session_ids = None;
    match recipient {
        Recipient::Channel(channel_name) => {
            let channel = channels::fetch_one(ctx, channel_name).await?;
            if !channel.can_write(session.privileges) {
                return Err(AppError::ChannelsUnauthorized);
            }
            recipient_channel = Some(channel_name);
            excluded_session_ids = Some(vec![session.session_id]);
        }
        Recipient::UserSession(receiver_session) => {
            if !receiver_session.is_publicly_visible()
                && !session.has_all_privileges(Privileges::AdminCaker)
            {
                return Err(AppError::Unauthorized);
            }
            recipient_id = Some(receiver_session.user_id);
        }
        Recipient::OfflineUser(username) => {
            let user = users::fetch_one_by_username(ctx, username).await?;
            if !user.privileges.is_publicly_visible()
                && !session.has_all_privileges(Privileges::AdminCaker)
            {
                return Err(AppError::Unauthorized);
            }
            recipient_id = Some(user.user_id);
            mark_as_unread = true;
        }
        Recipient::Bot => {
            recipient_id = Some(bot::BOT_ID);
        }
    }

    if commands::is_command_message(args.text) && recipient.can_process_commands() {
        tracing::warn!("Handle commands here");
    }

    messages::send(
        ctx,
        session.user_id,
        recipient_channel,
        recipient_id,
        args.text,
        mark_as_unread,
    )
    .await?;
    if let Some(stream_name) = recipient.get_message_stream() {
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
            excluded_session_ids,
            None,
        )
        .await?;
    }
    Ok(())
}

pub async fn send_bancho<C: Context>(
    ctx: &C,
    session: &Session,
    recipient: Recipient<'_>,
    message: IrcMessage<'_>,
) -> ServiceResult<Option<Vec<u8>>> {
    match send(ctx, session, recipient, message).await {
        Ok(()) => match recipient {
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
            _ => Ok(None),
        },
        Err(AppError::MessagesTooLong) => {
            let alert = Alert {
                message: "Your message was too long. It has not been sent.",
            };
            Ok(Some(alert.as_message().serialize()))
        }
        Err(AppError::MessagesUserAutoSilenced) => {
            let alert = Alert {
                message: "You have sent too many messages in a short period of time.",
            };
            Ok(Some(alert.as_message().serialize()))
        }
        Err(AppError::ChannelsUnauthorized) => {
            let alert = Alert {
                message: "You do not have permission to send messages to this channel.",
            };
            Ok(Some(alert.as_message().serialize()))
        }
        Err(AppError::Unauthorized) => {
            let alert = Alert {
                message: "You do not have permission to send messages to this user.",
            };
            Ok(Some(alert.as_message().serialize()))
        }
        Err(AppError::UsersNotFound) => {
            let alert = Alert {
                message: "This user does not exist.",
            };
            Ok(Some(alert.as_message().serialize()))
        }
        Err(e) => Err(e),
    }
}
