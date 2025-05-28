use crate::common::error::ServiceResult;

pub const COMMAND_PREFIX: &str = "!";

pub fn is_command_message(content: &str) -> bool {
    content.starts_with(COMMAND_PREFIX)
}

pub type CommandResult = ServiceResult<()>;

pub fn handle_command(_msg_content: &str) -> CommandResult {
    Ok(())
}
