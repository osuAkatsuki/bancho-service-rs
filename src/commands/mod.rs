mod command_handler;
mod from_args;

use bancho_protocol::messages::server::ChatMessage;
use bancho_protocol::structures::IrcMessage;
pub use command_handler::{Command, CommandRouter, CommandRouterInstance};
pub use from_args::FromCommandArgs;
use tracing::error;

pub mod misc;

use crate::commands;
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult};
use crate::entities::bot;
use crate::entities::channels::ChannelName;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::streams;

pub const COMMAND_PREFIX: &str = "!";
static COMMAND_ROUTER: CommandRouterInstance = commands![misc::alert_user];

pub type CommandResult = ServiceResult<()>;

pub fn is_command_message(content: &str) -> bool {
    content.starts_with(COMMAND_PREFIX)
}

pub struct CommandExecuteResult {
    forward_message: bool,
    read_privileges: Option<Privileges>,
}

impl Default for CommandExecuteResult {
    fn default() -> Self {
        Self {
            forward_message: true,
            read_privileges: None,
        }
    }
}

pub async fn handle_command<C: Context>(
    ctx: &C,
    sender: &Session,
    message_content: &str,
    recipient_channel: Option<ChannelName<'_>>,
) -> CommandExecuteResult {
    // Message does not start with command prefix, ignore
    let msg_content = match message_content.strip_prefix(COMMAND_PREFIX) {
        Some(msg_content) => msg_content,
        None => return Default::default(),
    };

    let mut parts = msg_content.splitn(2, ' ');
    let cmd_name = match parts.next() {
        Some(cmd_name) => cmd_name,
        // No command, ignore
        None => return Default::default(),
    };
    let args = parts.next();

    let command = COMMAND_ROUTER.get(cmd_name);
    match command {
        Some(command) => {
            let read_privileges = Some(command.required_privileges);
            let result = CommandExecuteResult {
                forward_message: true,
                read_privileges,
            };
            // TODO: maybe do this in a seperate task
            match command.handler().handle(ctx, sender, args).await {
                Ok(()) => result,
                Err(AppError::CommandsInvalidSyntax(syntax, _, typed)) => {
                    let (recipient, stream) = match recipient_channel {
                        None => (
                            sender.username.as_str(),
                            StreamName::User(sender.session_id),
                        ),
                        Some(ref channel_name) => {
                            (channel_name.to_bancho(), channel_name.get_message_stream())
                        }
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
                    let _ = streams::broadcast_message(
                        ctx,
                        stream,
                        ChatMessage(&syntax_message),
                        None,
                        read_privileges,
                    )
                    .await;
                    result
                }
                Err(e) => {
                    error!(
                        sender_id = sender.user_id,
                        message_content, "Error handling command: {e:?}"
                    );
                    result
                }
            }
        }
        None => Default::default(),
    }
}
