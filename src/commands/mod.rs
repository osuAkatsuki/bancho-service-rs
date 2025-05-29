mod command_handler;
mod from_args;

pub use command_handler::{Command, CommandRouter, CommandRouterInstance};
pub use from_args::FromCommandArgs;

pub mod misc;

use crate::commands;
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult};
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;

pub const COMMAND_PREFIX: &str = "!";
static COMMAND_ROUTER: CommandRouterInstance = commands![misc::alert_user];

pub type CommandResult = ServiceResult<String>;

pub fn is_command_message(content: &str) -> bool {
    content.starts_with(COMMAND_PREFIX)
}

pub enum CommandExecutionResult {
    /// No command found or message was not a command
    NoCommand,
    Success {
        response: String,
        forward_message: bool,
        read_privileges: Option<Privileges>,
    },
}

pub async fn handle_command<C: Context>(
    ctx: &C,
    sender: &Session,
    message_content: &str,
) -> ServiceResult<CommandExecutionResult> {
    // Message does not start with command prefix, ignore
    let msg_content = match message_content.strip_prefix(COMMAND_PREFIX) {
        Some(msg_content) => msg_content,
        None => return Ok(CommandExecutionResult::NoCommand),
    };

    let mut parts = msg_content.splitn(2, ' ');
    let cmd_name = match parts.next() {
        Some(cmd_name) => cmd_name,
        // No command, ignore
        None => return Ok(CommandExecutionResult::NoCommand),
    };
    let args = parts.next();

    let command = match COMMAND_ROUTER.get(cmd_name) {
        Some(command) => command,
        None => return Ok(CommandExecutionResult::NoCommand),
    };

    if let Some(required_privileges) = command.required_privileges {
        if !sender.has_all_privileges(required_privileges) {
            return Err(AppError::CommandsUnauthorized);
        }
    }

    let response = command.handler().handle(ctx, &sender, args).await?;
    Ok(CommandExecutionResult::Success {
        response,
        forward_message: command.forward_message,
        read_privileges: command.read_privileges,
    })
}
