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

// TODO: !mp addref
// TODO: !mp rmref
// TODO: !mp listref
// TODO: !mp make
// TODO: !mp close
// TODO: !mp lock
// TODO: !mp unlock
// TODO: !mp size
// TODO: !mp move
// TODO: !mp host
// TODO: !mp clearhost
// TODO: !mp start
// TODO: !mp invite
// TODO: !mp map
// TODO: !mp set
// TODO: !mp abort
// TODO: !mp kick
// TODO: !mp password
// TODO: !mp randompassword
// TODO: !mp mods
// TODO: !mp team
// TODO: !mp settings
// TODO: !mp scorev
// TODO: !mp help
// TODO: !mp link
// TODO: !mp timer
// TODO: !mp aborttimer
