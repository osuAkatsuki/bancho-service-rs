use crate::commands::CommandResult;
use crate::common::context::Context;
use crate::models::sessions::Session;
use crate::usecases::tillerino;
use bancho_service_macros::command;

#[command("map")]
pub async fn edit_map<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let _last_np = tillerino::fetch_last_np(ctx, sender.session_id).await?;
    Ok("Please /np a map first!".to_owned())
}

#[command("addbn")]
pub async fn add_bn<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("removebn")]
pub async fn remove_bn<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("moderated")]
pub async fn set_moderated<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("kick")]
pub async fn kick<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("silence")]
pub async fn silence_user<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("unsilence")]
pub async fn unsilence_user<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("ban")]
pub async fn ban_user<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("unban")]
pub async fn unban_user<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("restrict")]
pub async fn restrict_user<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("unrestrict")]
pub async fn unrestrict_user<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("freeze")]
pub async fn freeze_user<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("unfreeze")]
pub async fn unfreeze_user<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}

#[command("whitelist")]
pub async fn whitelist_user<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(todo!())
}
