mod command_handler;
mod from_args;
pub mod misc;
pub mod mp;
pub mod staff;
pub mod system;
use command_handler::CommandHandlerProxy;

pub use command_handler::{
    Command, CommandProperties, CommandRouter, CommandRouterFactory, CommandRouterInstance,
    RegisteredCommand,
};
pub use from_args::FromCommandArgs;

use crate::commands;
use crate::common::context::Context;
use crate::common::error::ServiceResult;
use crate::models::messages::Recipient;
use crate::models::performance::PerformanceRequestArgs;
use crate::models::sessions::Session;
use crate::models::tillerino::NowPlayingMessage;
use crate::usecases::{performance, tillerino};
use std::sync::LazyLock;

pub const COMMAND_PREFIX: &str = "!";

static COMMAND_ROUTER: CommandRouterInstance = LazyLock::new(commands![
    include = [
        "mp" => mp::COMMANDS,
        "system" => system::COMMANDS,
    ],
    misc::alert_all,
    misc::alert_user,
    misc::announce,
    misc::help,
    misc::last_user_score,
    misc::map_mirror,
    misc::report_user,
    misc::roll,
    misc::pp_with,

    staff::add_bn,
    staff::ban_user,
    staff::edit_map,
    staff::freeze_user,
    staff::kick,
    staff::remove_bn,
    staff::restrict_user,
    staff::silence_user,
    staff::unban_user,
    staff::unfreeze_user,
    staff::unrestrict_user,
    staff::unsilence_user,
    staff::whitelist_user,
]);

#[derive(Debug)]
pub struct CommandResponse {
    pub answer: Option<String>,
    pub properties: CommandProperties,
}

impl Default for CommandResponse {
    fn default() -> CommandResponse {
        CommandResponse {
            answer: None,
            properties: CommandProperties::default(),
        }
    }
}

pub type CommandResult = ServiceResult<Option<String>>;

pub fn is_command_message(content: &str) -> bool {
    content.starts_with(COMMAND_PREFIX)
}

pub async fn handle_command<C: Context>(
    ctx: &C,
    sender: &Session,
    message_content: &str,
) -> ServiceResult<Option<CommandResponse>> {
    // Message does not start with command prefix, ignore
    let cmd_message = message_content.strip_prefix(COMMAND_PREFIX);
    COMMAND_ROUTER.handle(ctx, sender, cmd_message).await
}

pub async fn try_handle_command<C: Context>(
    ctx: &C,
    session: &Session,
    message_content: &str,
    recipient: &Recipient<'_>,
) -> ServiceResult<Option<CommandResponse>> {
    if is_command_message(message_content) {
        handle_command(ctx, session, message_content).await
    } else if let Some(np_message) = NowPlayingMessage::parse(message_content) {
        tracing::info!("/np message received: {np_message:?}");
        let np = tillerino::save_np(ctx, session.session_id, np_message).await?;
        match recipient.is_bot() {
            true => {
                let response =
                    performance::fetch_pp_message(PerformanceRequestArgs::from(np)).await?;
                Ok(Some(CommandResponse {
                    answer: Some(response),
                    ..Default::default()
                }))
            }
            false => Ok(None),
        }
    } else {
        Ok(None)
    }
}
