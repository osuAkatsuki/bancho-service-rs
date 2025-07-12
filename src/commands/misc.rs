use crate::commands::{COMMAND_PREFIX, COMMAND_ROUTER, CommandResult};
use crate::common::context::Context;
use crate::entities::bot;
use crate::models::performance::PerformanceRequestArgs;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{multiplayer, performance, sessions, spectators, streams, tillerino};
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

#[command("mirror")]
pub async fn map_mirror<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    match spectators::fetch_spectating(ctx, sender.session_id).await? {
        Some(_) => todo!(),
        None => {}
    }

    match multiplayer::fetch_session_match_id(ctx, sender.session_id).await? {
        Some(_) => todo!(),
        None => {}
    }
    Ok(todo!())
}

#[command("with")]
pub async fn pp_with<C: Context>(ctx: &C, sender: &Session, args: String) -> CommandResult {
    let last_np = tillerino::fetch_last_np(ctx, sender.session_id).await?;
    match last_np {
        Some(last_np) => {
            let request = PerformanceRequestArgs::from_extra(last_np, &args)?;
            Ok(performance::fetch_pp_message(request).await?)
        }
        None => Ok("You haven't /np'ed a map yet! Please use /np".to_owned()),
    }
}

#[command("last")]
pub async fn last_user_score<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("report")]
pub async fn report_user<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("overwrite")]
pub async fn overwrite_score<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}
