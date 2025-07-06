mod command_handler;
mod from_args;

use command_handler::CommandHandlerProxy;
pub use command_handler::{
    Command, CommandProperties, CommandRouter, CommandRouterFactory, CommandRouterInstance,
    RegisteredCommand,
};
use std::sync::LazyLock;

pub use from_args::FromCommandArgs;

pub mod misc;
pub mod mp;
pub mod staff;

use crate::commands;
use crate::common::context::Context;
use crate::common::error::ServiceResult;
use crate::models::sessions::Session;

pub const COMMAND_PREFIX: &str = "!";

static COMMAND_ROUTER: CommandRouterInstance = LazyLock::new(commands![
    include = [
        "mp" => mp::COMMANDS,
    ],
    misc::alert_all,
    misc::alert_user,
    misc::announce,
    misc::help,
    misc::roll,
]);

pub struct CommandResponse {
    pub answer: Option<String>,
    pub properties: CommandProperties,
}
impl Default for CommandResponse {
    fn default() -> CommandResponse {
        CommandResponse {
            answer: None,
            properties: CommandProperties {
                name: "",
                forward_message: true,
                required_privileges: None,
                read_privileges: None,
            },
        }
    }
}
pub type CommandResult = ServiceResult<String>;

pub fn is_command_message(content: &str) -> bool {
    content.starts_with(COMMAND_PREFIX)
}

pub async fn handle_command<C: Context>(
    ctx: &C,
    sender: &Session,
    message_content: &str,
) -> ServiceResult<CommandResponse> {
    // Message does not start with command prefix, ignore
    let cmd_message = message_content.strip_prefix(COMMAND_PREFIX);
    COMMAND_ROUTER.handle(ctx, sender, cmd_message).await
}
