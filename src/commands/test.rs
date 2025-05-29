use crate::commands::CommandResult;
use crate::common::context::Context;
use crate::models::sessions::Session;
use bancho_service_macros::command;

#[command("test")]
pub async fn test_command<C: Context + ?Sized>(
    _ctx: &C,
    session: &Session,
    args: Option<String>,
) -> CommandResult {
    tracing::info!(user_id = session.user_id, "Testing command; args={args:?}");
    Ok(())
}
