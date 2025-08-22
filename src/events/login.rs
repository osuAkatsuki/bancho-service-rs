use crate::api::RequestContext;
use crate::common::error::AppError;
use crate::entities::bot;
use crate::entities::channels::ChannelName;
use crate::models::bancho::{BanchoResponse, LoginArgs, LoginError};
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{
    bancho_settings, channels, messages, presences, relationships, sessions, streams,
};
use bancho_protocol::concat_messages;
use bancho_protocol::messages::server::{
    Alert, ChannelInfo, ChannelInfoEnd, ChannelJoinSuccess, ChatMessage, FriendsList, LoginResult,
    ProtocolVersion, SilenceEnd, UserPresenceBundle, UserPrivileges,
};
use bancho_protocol::messages::{Message, MessageArgs};
use bancho_protocol::serde::BinarySerialize;
use bancho_protocol::serde::osu_types::PrefixedVec;
use bancho_protocol::structures::{IrcMessage, Privileges};
use tracing::{error, info};

const WELCOME_MESSAGE: &str = r#"
             Welcome to Akatsuki!
             Running banchus v0.1
 "#; // This space is needed for osu! to render the line

fn login_error(e: AppError) -> BanchoResponse {
    let login_error = match e {
        AppError::SessionsInvalidCredentials => LoginError::InvalidCredentials,
        AppError::ClientTooOld => LoginError::OldVersion,
        AppError::SessionsLoginForbidden => LoginError::Banned,
        AppError::SessionsLimitReached => LoginError::OldVersion,
        _ => LoginError::UnexpectedError,
    };
    let data = concat_messages!(
        Alert {
            message: e.message()
        },
        LoginResult {
            user_id: login_error as _,
        }
    );
    BanchoResponse::error_raw(None, data)
}

pub async fn handle(ctx: &RequestContext, args: LoginArgs) -> BanchoResponse {
    match bancho_settings::in_maintenance_mode(ctx).await {
        Ok(true) => return login_error(AppError::MaintenanceModeEnabled),
        Err(e) => return login_error(e),
        _ => {}
    }

    let (session, presence) = match sessions::create(ctx, args).await {
        Ok(res) => res,
        Err(e) => return login_error(e),
    };
    let user_panel = presence.user_panel();
    if session.is_publicly_visible() {
        match streams::broadcast_data(ctx, StreamName::Main, &user_panel, None, None).await {
            Ok(_) => (),
            Err(e) => error!("Failed to broadcast user panel: {e:?}"),
        };
    }

    let friends: Vec<i32> = match relationships::fetch_friends(ctx, session.user_id).await {
        Ok(friends) => friends.into_iter().map(|r| r.friend_id as i32).collect(),
        Err(e) => {
            error!("Failed fetching friends: {e:?}");
            vec![]
        }
    };

    let mut response = vec![
        concat_messages! {
            LoginResult{ user_id: session.user_id as _ },
            ProtocolVersion { version: bancho_protocol::PROTOCOL_VERSION },
            UserPrivileges { privileges: session.privileges.to_bancho() | Privileges::Supporter },
            ChannelInfoEnd,
            Alert{ message: WELCOME_MESSAGE },
            FriendsList::from(friends),
        },
        user_panel,
        bot::user_panel(),
    ];
    let silence_left = session.silence_left();
    if silence_left != 0 {
        response.push(Message::serialize(SilenceEnd {
            seconds_left: silence_left as _,
        }));
    }

    let _ = streams::join(
        ctx,
        session.session_id,
        StreamName::User(session.session_id),
    )
    .await;
    let _ = streams::join(ctx, session.session_id, StreamName::Main).await;
    join_special_channel(ctx, &mut response, &session, "#osu").await;
    join_special_channel(ctx, &mut response, &session, "#announce").await;

    if session.privileges.is_donor() {
        let _ = streams::join(ctx, session.session_id, StreamName::Donator).await;
        join_special_channel(ctx, &mut response, &session, "#plus").await;
    }

    if session.privileges.is_staff() {
        let _ = streams::join(ctx, session.session_id, StreamName::Staff).await;
        join_special_channel(ctx, &mut response, &session, "#staff").await;
    }

    if session.privileges.is_developer() {
        let _ = streams::join(ctx, session.session_id, StreamName::Dev).await;
        join_special_channel(ctx, &mut response, &session, "#devlog").await;
    }

    match channels::fetch_all(ctx).await {
        Ok(channels) => {
            for channel in channels {
                if !channel.can_read(session.privileges) {
                    continue;
                }

                let member_count = channels::member_count(ctx, ChannelName::Chat(&channel.name))
                    .await
                    .unwrap_or_else(|e| {
                        error!("Failed to fetch channel member count: {e:?}");
                        0
                    });
                let info = ChannelInfo {
                    name: &channel.name,
                    topic: &channel.description,
                    user_count: member_count as _,
                };
                response.push(info.as_message().serialize());
            }
        }
        Err(e) => error!("Failed to fetch channels during login: {e:?}"),
    }

    match presences::fetch_user_ids(ctx).await {
        Ok(user_ids) => {
            let presence_bundle = UserPresenceBundle {
                user_ids: PrefixedVec::from(user_ids),
            };
            response.push(presence_bundle.as_message().serialize());
        }
        Err(e) => error!("Failed to fetch presences during login: {e:?}"),
    }

    match messages::fetch_unread_messages(ctx, session.user_id).await {
        Ok(unread_messages) => {
            match messages::mark_all_read(ctx, session.user_id).await {
                Ok(()) => {}
                Err(e) => error!("Failed to mark all messages as read: {e:?}"),
            }

            let unread_messages = unread_messages.map(|msg| {
                let msg = IrcMessage {
                    sender: &msg.sender_name,
                    text: &msg.content,
                    recipient: &session.username,
                    sender_id: msg.sender_id as _,
                };
                Message::serialize(ChatMessage(&msg))
            });
            response.extend(unread_messages);
        }
        Err(e) => error!("Failed to fetch unread messages during login: {e:?}"),
    }

    info!(
        user_id = session.user_id,
        username = presence.username,
        "User logged in."
    );
    BanchoResponse::ok(session.session_id, response.concat())
}

async fn join_special_channel<'a>(
    ctx: &RequestContext,
    response: &mut Vec<Vec<u8>>,
    session: &Session,
    channel_name: &'a str,
) {
    match channels::join(ctx, &session, ChannelName::Chat(channel_name)).await {
        Ok(_) => {
            let success = ChannelJoinSuccess { name: channel_name };
            response.push(success.as_message().serialize());
        }
        Err(e) => {
            error!("Failed to join special channel: {e:?}");
        }
    }
}
