use crate::commands;
use crate::commands::{CommandResult, CommandRouterFactory};
use crate::common::context::Context;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::usecases::bancho_settings;
use bancho_service_macros::command;

pub static COMMANDS: CommandRouterFactory = commands![maintenance];

#[command(
    "maintenance",
    required_privileges = Privileges::AdminCaker,
)]
pub async fn maintenance<C: Context>(ctx: &C, _sender: &Session) -> CommandResult {
    let is_active = bancho_settings::toggle_maintenance(ctx).await?;
    // TODO: kick all non-admin users unless they're in a multiplayer match
    let on_off = match is_active {
        true => "on",
        false => "off",
    };
    Ok(Some(format!("Turned {on_off} maintenance mode.")))
}
