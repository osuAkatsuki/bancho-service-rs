mod command_handler;
mod from_args;

pub use command_handler::{Command, CommandRouter, CommandRouterInstance};
pub use from_args::FromCommandArgs;

pub mod test;

use crate::commands;
use crate::common::context::Context;
use crate::common::error::ServiceResult;
use crate::models::sessions::Session;

pub const COMMAND_PREFIX: &str = "!";
static COMMAND_ROUTER: CommandRouterInstance = commands![test::test_command];

pub type CommandResult = ServiceResult<()>;

pub fn is_command_message(content: &str) -> bool {
    content.starts_with(COMMAND_PREFIX)
}

pub async fn handle_command<C: Context>(
    ctx: &C,
    sender: &Session,
    message_content: &str,
) -> CommandResult {
    // Message does not start with command prefix, ignore
    let msg_content = match message_content.strip_prefix(COMMAND_PREFIX) {
        Some(msg_content) => msg_content,
        None => return Ok(()),
    };

    let mut parts = msg_content.splitn(2, ' ');
    let cmd_name = match parts.next() {
        Some(cmd_name) => cmd_name,
        // No command, ignore
        None => return Ok(()),
    };
    let args = parts.next();

    let handler = COMMAND_ROUTER.get_handler(cmd_name);
    match handler {
        Some(handler) => {
            // TODO: maybe do this in a seperate task
            handler.handle(ctx, sender, args).await
        }
        None => Ok(()),
    }
}
