use crate::commands;
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::entities::bot;
use crate::entities::channels::ChannelName;
use crate::models::messages::{Message, MessageSendResult, Recipient};
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::messages;
use crate::usecases::{channels, relationships, users};
use chrono::{TimeDelta, Utc};
use tracing::error;

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
}

impl Default for RecipientInfo<'_> {
    fn default() -> Self {
        Self {
            recipient_channel: None,
            recipient_id: None,
            mark_as_unread: false,
        }
    }
}

pub async fn check_spam<C: Context>(ctx: &C, session: &mut Session) -> ServiceResult<()> {
    let message_count =
        messages::message_count(ctx, session.user_id, CHAT_SPAM_RATE_INTERVAL).await?;
    if message_count < CHAT_SPAM_RATE {
        return Ok(());
    }

    session.silence_end = Some(Utc::now() + TimeDelta::seconds(CHAT_TIMEOUT_SECONDS));
    users::silence_user(
        ctx,
        session.user_id,
        CHAT_TIMEOUT_REASON,
        CHAT_TIMEOUT_SECONDS,
    )
    .await?;
    Err(AppError::MessagesUserAutoSilenced)
}

pub async fn send<C: Context>(
    ctx: &C,
    session: &mut Session,
    recipient: &Recipient<'_>,
    message_content: &str,
) -> ServiceResult<MessageSendResult> {
    if !session.is_publicly_visible() {
        return Err(AppError::InteractionBlocked);
    }

    if session.is_silenced() {
        return Err(AppError::MessagesUserSilenced);
    }

    let message_content = message_content.trim();
    if message_content.is_empty() && message_content.len() > 500 {
        return Err(AppError::MessagesInvalidLength);
    }

    check_spam(ctx, session).await?;
    let recipient_info = get_recipient_info(ctx, session, &recipient).await?;
    let message = messages::send(
        ctx,
        session.user_id,
        &session.username,
        recipient_info.recipient_channel,
        recipient_info.recipient_id,
        message_content,
        recipient_info.mark_as_unread,
    )
    .await
    .map(Message::from)?;

    let response = commands::try_handle_command(ctx, session, message_content, &recipient).await?;
    Ok(MessageSendResult { message, response })
}

async fn get_recipient_info<'a, C: Context>(
    ctx: &C,
    sender: &Session,
    recipient: &Recipient<'a>,
) -> ServiceResult<RecipientInfo<'a>> {
    match recipient {
        Recipient::Channel(channel_name) => {
            let channel = channels::fetch_one(ctx, *channel_name).await?;
            if !channel.can_write(sender.privileges) {
                return Err(AppError::ChannelsUnauthorized);
            }

            Ok(RecipientInfo {
                recipient_channel: Some(*channel_name),
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
