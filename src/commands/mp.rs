use crate::commands;
use crate::commands::{CommandResult, CommandRouterFactory};
use crate::common::context::Context;
use crate::models::sessions::Session;
use bancho_service_macros::command;

pub static COMMANDS: CommandRouterFactory = commands![host];

#[command("host")]
pub async fn host<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    let response = format!("Transferred host to user");
    Ok(response)
}
