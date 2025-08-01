use crate::api::RequestContext;
use crate::common::error::AppError;
use crate::entities::bot;
use crate::events::EventResult;
use crate::models::messages::Recipient;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{channels, messages, sessions, streams};
use bancho_protocol::messages::MessageArgs;
use bancho_protocol::messages::client::{PrivateChatMessage, PublicChatMessage};
use bancho_protocol::messages::server::ChatMessage;
use bancho_protocol::serde::BinarySerialize;
use bancho_protocol::structures::IrcMessage;

// TODO: simplify all this lol
pub async fn public_chat_message(
    ctx: &RequestContext,
    session: &mut Session,
    args: PublicChatMessage<'_>,
) -> EventResult {
    let channel_name = channels::get_channel_name(ctx, session, &args.message.recipient).await?;
    let recipient = Recipient::Channel(channel_name);

    let result = messages::send(ctx, session, &recipient, args.message.text).await?;
    match result.response {
        Some(cmd_response) => {
            let bot_response = cmd_response.answer.map(|answer| {
                let bot_response_msg = IrcMessage {
                    sender: bot::BOT_NAME,
                    sender_id: bot::BOT_ID as _,
                    text: &answer,
                    recipient: args.message.recipient,
                };
                ChatMessage(&bot_response_msg).as_message().serialize()
            });

            match cmd_response.properties.forward_message {
                true => {
                    let message_stream = channel_name.get_message_stream();
                    let msg = IrcMessage {
                        sender: &session.username,
                        text: &result.message.content,
                        recipient: args.message.recipient,
                        sender_id: session.user_id as _,
                    };
                    streams::broadcast_message(
                        ctx,
                        message_stream,
                        ChatMessage(&msg),
                        Some(vec![session.session_id]),
                        cmd_response.properties.read_privileges,
                    )
                    .await?;

                    if let Some(bot_response) = bot_response {
                        streams::broadcast_data(
                            ctx,
                            message_stream,
                            &bot_response,
                            None,
                            cmd_response.properties.read_privileges,
                        )
                        .await?;
                    }
                    Ok(None)
                }
                false => Ok(bot_response),
            }
        }
        None => {
            let message_stream = channel_name.get_message_stream();
            let msg = IrcMessage {
                sender: &session.username,
                text: &result.message.content,
                recipient: args.message.recipient,
                sender_id: session.user_id as _,
            };
            streams::broadcast_message(
                ctx,
                message_stream,
                ChatMessage(&msg),
                Some(vec![session.session_id]),
                None,
            )
            .await?;
            Ok(None)
        }
    }
}

pub async fn private_chat_message(
    ctx: &RequestContext,
    session: &mut Session,
    args: PrivateChatMessage<'_>,
) -> EventResult {
    let recipient_name = args.message.recipient;
    let recipient = match recipient_name == bot::BOT_NAME {
        true => Recipient::Bot,
        false => {
            let recipient_session = sessions::fetch_primary_by_username(ctx, recipient_name).await;
            match recipient_session {
                Ok(recipient_session) => Recipient::UserSession(recipient_session),
                Err(AppError::SessionsNotFound) => Recipient::OfflineUser(recipient_name),
                Err(e) => return Err(e),
            }
        }
    };

    let result = messages::send(ctx, session, &recipient, args.message.text).await?;
    match recipient {
        Recipient::Channel(_) => unreachable!(),
        Recipient::UserSession(recipient_session) => match result.response {
            Some(cmd_response) => {
                let bot_response = cmd_response.answer.map(|answer| {
                    let bot_response_msg = IrcMessage {
                        sender: bot::BOT_NAME,
                        sender_id: bot::BOT_ID as _,
                        text: &answer,
                        recipient: &session.username,
                    };
                    ChatMessage(&bot_response_msg).as_message().serialize()
                });
                let forward_message = cmd_response.properties.forward_message
                    && cmd_response
                        .properties
                        .read_privileges
                        .is_none_or(|read_privileges| {
                            recipient_session.has_all_privileges(read_privileges)
                        });

                match forward_message {
                    true => {
                        let msg = IrcMessage {
                            sender: &session.username,
                            sender_id: session.user_id as _,
                            text: &result.message.content,
                            recipient: &recipient_session.username,
                        };
                        let message_stream = StreamName::User(recipient_session.session_id);
                        streams::broadcast_message(
                            ctx,
                            message_stream,
                            ChatMessage(&msg),
                            None,
                            None,
                        )
                        .await?;
                        Ok(bot_response)
                    }
                    false => Ok(bot_response),
                }
            }
            None => {
                let msg = IrcMessage {
                    sender: &session.username,
                    sender_id: session.user_id as _,
                    text: &result.message.content,
                    recipient: &recipient_session.username,
                };
                let message_stream = StreamName::User(recipient_session.session_id);
                streams::broadcast_message(ctx, message_stream, ChatMessage(&msg), None, None)
                    .await?;
                Ok(None)
            }
        },
        Recipient::Bot => {
            if let Some(response) = result.response
                && let Some(bot_reply) = response.answer
            {
                let bot_reply_msg = IrcMessage {
                    sender: bot::BOT_NAME,
                    sender_id: bot::BOT_ID as _,
                    text: &bot_reply,
                    recipient: &session.username,
                };
                Ok(Some(ChatMessage(&bot_reply_msg).as_message().serialize()))
            } else {
                Ok(None)
            }
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
    }
}

/*
TODO: handle invalid syntax (probably rewrite error message system lol)
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
*/
