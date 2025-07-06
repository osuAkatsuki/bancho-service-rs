use crate::commands::{COMMAND_PREFIX, COMMAND_ROUTER, CommandResult};
use crate::common::context::Context;
use crate::entities::bot;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{sessions, streams};
use bancho_protocol::messages::server::{Alert, ChatMessage};
use bancho_protocol::structures::IrcMessage;
use bancho_service_macros::{FromCommandArgs, command};

#[derive(Debug, FromCommandArgs)]
pub struct AlertUserArgs {
    pub username: String,
    pub message: String,
}

#[command("help")]
pub async fn help<C: Context>(_ctx: &C, sender: &Session) -> CommandResult {
    let mut response = "Available commands:\n".to_owned();
    for (name, cmd) in COMMAND_ROUTER.commands.iter() {
        match cmd.properties.required_privileges {
            Some(required_privileges) if sender.has_all_privileges(required_privileges) => {
                response.push_str(COMMAND_PREFIX);
                response.push_str(name);
                response.push('\n');
            }
            None => {
                response.push_str(COMMAND_PREFIX);
                response.push_str(name);
                response.push('\n');
            }
            _ => {}
        }
    }

    Ok(response)
}

#[command(
    "alert",
    required_privileges = Privileges::AdminSendAlerts,
)]
pub async fn alert_user<C: Context>(
    ctx: &C,
    _sender: &Session,
    args: AlertUserArgs,
) -> CommandResult {
    let session = sessions::fetch_primary_by_username(ctx, &args.username).await?;
    let alert = Alert {
        message: &args.message,
    };
    streams::broadcast_message(ctx, StreamName::User(session.session_id), alert, None, None)
        .await?;
    Ok("Alert sent successfully.".to_owned())
}

#[command(
    "alertall",
    required_privileges = Privileges::AdminSendAlerts,
)]
pub async fn alert_all<C: Context>(ctx: &C, _sender: &Session, message: String) -> CommandResult {
    let alert = Alert { message: &message };
    streams::broadcast_message(ctx, StreamName::Main, alert, None, None).await?;
    Ok("Alert sent successfully.".to_owned())
}

#[command(
    "announce",
    required_privileges = Privileges::AdminSendAlerts,
)]
pub async fn announce<C: Context>(ctx: &C, _sender: &Session, message: String) -> CommandResult {
    let msg = IrcMessage {
        sender_id: bot::BOT_ID as _,
        sender: bot::BOT_NAME,
        recipient: "#announce",
        text: &message,
    };
    streams::broadcast_message(ctx, StreamName::Main, ChatMessage(&msg), None, None).await?;
    Ok("Announcement sent successfully.".to_owned())
}

const MAX_ROLL: i32 = 1_000_000;

#[command("roll")]
pub async fn roll<C: Context>(_ctx: &C, sender: &Session, max_roll: Option<i32>) -> CommandResult {
    let max_roll = max_roll.unwrap_or(MAX_ROLL).min(MAX_ROLL).max(1);
    let result = rand::random_range(1..=max_roll);
    let response = format!("{} rolls {result} points!", sender.username);
    Ok(response)
}

// TODO: !addbn
// TODO: !removebn
// TODO: !roll
// TODO: !moderated
// TODO: !kick
// TODO: !silence
// TODO: !unsilence
// TODO: !ban
// TODO: !unban
// TODO: !restrict
// TODO: !unrestrict
// TODO: !system maintenance
// TODO: !mapdl
// TODO: !with
// TODO: !last
// TODO: !report
// TODO: !freeze
// TODO: !unfreeze
// TODO: !map
// TODO: !announce
// TODO: !whitelist
// TODO: !overwrite
