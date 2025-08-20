use crate::commands::{CommandResult, COMMAND_ROUTER, CommandRouterFactory};
use crate::common::context::Context;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::usecases::{channels, sessions, streams, users};
use crate::repositories::streams::StreamName;
use bancho_protocol::messages::server::{Alert, ChatMessage, LoginBanned, UserLogout};
use bancho_service_macros::{command, FromCommandArgs};
use chrono::TimeDelta;
use std::str::FromStr;

pub static COMMANDS: CommandRouterFactory = commands![
    add_bn,
    ban_user,
    edit_map,
    freeze_user,
    kick,
    moderated,
    remove_bn,
    restrict_user,
    silence_user,
    unban_user,
    unfreeze_user,
    unrestrict_user,
    unsilence_user,
    whitelist_user,
];

#[derive(Debug, FromCommandArgs)]
pub struct AddBnArgs {
    pub username: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct RemoveBnArgs {
    pub username: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct ModeratedArgs {
    pub enable: Option<String>,
}

#[derive(Debug, FromCommandArgs)]
pub struct KickArgs {
    pub username: String,
    pub reason: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct SilenceArgs {
    pub username: String,
    pub amount: i32,
    pub unit: String,
    pub reason: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct UnsilenceArgs {
    pub username: String,
    pub reason: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct BanArgs {
    pub username: String,
    pub reason: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct UnbanArgs {
    pub username: String,
    pub reason: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct RestrictArgs {
    pub username: String,
    pub reason: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct UnrestrictArgs {
    pub username: String,
    pub reason: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct FreezeArgs {
    pub username: String,
    pub reason: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct UnfreezeArgs {
    pub username: String,
    pub reason: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct WhitelistArgs {
    pub username: String,
    pub bit: i32,
    pub reason: String,
}

#[derive(Debug, FromCommandArgs)]
pub struct EditMapArgs {
    pub action: String,
    pub scope: String,
}

#[command(
    "map",
    required_privileges = Privileges::AdminManageBeatmaps,
)]
pub async fn edit_map<C: Context>(ctx: &C, sender: &Session, args: EditMapArgs) -> CommandResult {
    let last_np = crate::usecases::tillerino::fetch_last_np(ctx, sender.session_id).await?;
    if last_np.is_none() {
        return Ok(Some("Please /np a map first!".to_owned()));
    }

    // TODO: Implement map ranking/unranking logic
    Ok(Some("Map editing functionality not yet implemented.".to_owned()))
}

#[command(
    "addbn",
    required_privileges = Privileges::AdminManageNominators,
)]
pub async fn add_bn<C: Context>(ctx: &C, sender: &Session, args: AddBnArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Add BN privileges
    let new_privileges = target_user.privileges
        | Privileges::AdminManageBeatmaps
        | Privileges::Donator
        | Privileges::AkatsukiPlus;

    users::update_user_privileges(ctx, target_user.user_id, new_privileges).await?;

    // Update all user sessions
    let target_sessions = sessions::fetch_by_username(ctx, &args.username).await?;
    for session in target_sessions {
        let mut updated_session = session;
        updated_session.privileges = new_privileges;
        sessions::update(ctx, updated_session).await?;
    }

    // TODO: Add BN badge
    // TODO: Set donor expiry

    Ok(Some(format!("{} has given BN to {}.", sender.username, args.username)))
}

#[command(
    "removebn",
    required_privileges = Privileges::AdminManageNominators,
)]
pub async fn remove_bn<C: Context>(ctx: &C, sender: &Session, args: RemoveBnArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Remove BN privileges
    let new_privileges = target_user.privileges
        & !Privileges::AdminManageBeatmaps
        & !Privileges::Donator
        & !Privileges::AkatsukiPlus;

    users::update_user_privileges(ctx, target_user.user_id, new_privileges).await?;

    // Update all user sessions
    let target_sessions = sessions::fetch_by_username(ctx, &args.username).await?;
    for session in target_sessions {
        let mut updated_session = session;
        updated_session.privileges = new_privileges;
        sessions::update(ctx, updated_session).await?;
    }

    // TODO: Remove BN badge
    // TODO: Set donor expiry to 0

    Ok(Some(format!("{} has removed BN from {}.", sender.username, args.username)))
}

#[command(
    "moderated",
    required_privileges = Privileges::AdminChatMod,
)]
pub async fn set_moderated<C: Context>(ctx: &C, sender: &Session, args: ModeratedArgs) -> CommandResult {
    // This command needs to know which channel it's being used in
    // For now, we'll assume it's being used in a public channel
    // TODO: Get the actual channel name from the context

    let enable = match args.enable.as_deref() {
        Some("off") => false,
        _ => true,
    };

    // TODO: Get channel name from context and update moderated status
    // channels::update_moderated_status(ctx, channel_name, enable).await?;

    // TODO: Log the action

    let status = if enable { "now" } else { "no longer" };
    Ok(Some(format!("This channel is {} in moderated mode!", status)))
}

#[command(
    "kick",
    required_privileges = Privileges::AdminKickUsers,
)]
pub async fn kick<C: Context>(ctx: &C, sender: &Session, args: KickArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;
    let target_sessions = sessions::fetch_by_username(ctx, &args.username).await?;

    let mut session_count = 0;
    for session in target_sessions {
        sessions::delete(ctx, &session).await?;
        session_count += 1;
    }

    if session_count == 0 {
        return Ok(Some("Target not online.".to_owned()));
    }

    // TODO: Log the action

    Ok(Some(format!("{} has been kicked from the server.", args.username)))
}

#[command(
    "ban",
    required_privileges = Privileges::AdminManagePrivileges,
)]
pub async fn ban_user<C: Context>(ctx: &C, sender: &Session, args: BanArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Ban the user (remove login privileges)
    users::ban_user(ctx, target_user.user_id).await?;

    // Send ban packet to online sessions
    let target_sessions = sessions::fetch_by_username(ctx, &args.username).await?;
    for session in target_sessions {
        let ban_packet = LoginBanned;
        streams::broadcast_message(ctx, StreamName::User(session.session_id), ban_packet, None, None).await?;
    }

    // TODO: Log the action

    Ok(Some(format!("{} has been banned.", args.username)))
}

#[command(
    "unban",
    required_privileges = Privileges::AdminManagePrivileges,
)]
pub async fn unban_user<C: Context>(ctx: &C, sender: &Session, args: UnbanArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Unban the user (restore login privileges)
    users::unban_user(ctx, target_user.user_id).await?;

    // TODO: Log the action

    Ok(Some(format!("{} has been unbanned.", args.username)))
}

#[command(
    "restrict",
    required_privileges = Privileges::AdminManageBans,
)]
pub async fn restrict_user<C: Context>(ctx: &C, sender: &Session, args: RestrictArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Restrict the user (remove publicly visible privileges)
    users::restrict_user(ctx, target_user.user_id).await?;

    // Notify online sessions
    let target_sessions = sessions::fetch_by_username(ctx, &args.username).await?;
    for session in target_sessions {
        // TODO: Send restriction notification packet
    }

    // TODO: Log the action

    Ok(Some(format!("{} has been restricted.", args.username)))
}

#[command(
    "unrestrict",
    required_privileges = Privileges::AdminManageBans,
)]
pub async fn unrestrict_user<C: Context>(ctx: &C, sender: &Session, args: UnrestrictArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Unrestrict the user (restore publicly visible privileges)
    users::unrestrict_user(ctx, target_user.user_id).await?;

    // TODO: Log the action

    Ok(Some(format!("{} has been unrestricted.", args.username)))
}

#[command(
    "freeze",
    required_privileges = Privileges::AdminFreezeUsers,
)]
pub async fn freeze_user<C: Context>(ctx: &C, sender: &Session, args: FreezeArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Check if user is already frozen
    if target_user.frozen {
        return Ok(Some("That user is already frozen.".to_owned()));
    }

    // Freeze the user
    users::freeze_user(ctx, target_user.user_id, &args.reason).await?;

    // TODO: Log the action

    Ok(Some(format!("Froze {}.", args.username)))
}

#[command(
    "unfreeze",
    required_privileges = Privileges::AdminFreezeUsers,
)]
pub async fn unfreeze_user<C: Context>(ctx: &C, sender: &Session, args: UnfreezeArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Check if user is frozen
    if !target_user.frozen {
        return Ok(Some("That user is not frozen.".to_owned()));
    }

    // Unfreeze the user
    users::unfreeze_user(ctx, target_user.user_id).await?;

    // TODO: Log the action

    Ok(Some(format!("Unfroze {}.", args.username)))
}

#[command(
    "whitelist",
    required_privileges = Privileges::AdminManageUsers,
)]
pub async fn whitelist_user<C: Context>(ctx: &C, sender: &Session, args: WhitelistArgs) -> CommandResult {
    if !(0..4).contains(&args.bit) {
        return Ok(Some("Invalid bit.".to_owned()));
    }

    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Update whitelist status
    users::update_user_whitelist(ctx, target_user.user_id, args.bit).await?;

    // Update online sessions
    let target_sessions = sessions::fetch_by_username(ctx, &args.username).await?;
    for session in target_sessions {
        // TODO: Update session whitelist status
    }

    // TODO: Log the action

    Ok(Some(format!("{}'s Whitelist Status has been set to {}.", args.username, args.bit)))
}

#[command(
    "silence",
    required_privileges = Privileges::AdminSilenceUsers,
)]
pub async fn silence_user<C: Context>(ctx: &C, sender: &Session, args: SilenceArgs) -> CommandResult {
    // Calculate silence seconds
    let silence_seconds = match args.unit.as_str() {
        "s" => args.amount,
        "m" => args.amount * 60,
        "h" => args.amount * 3600,
        "d" => args.amount * 86400,
        "w" => args.amount * 604800,
        _ => return Ok(Some("Invalid time unit (s/m/h/d/w).".to_owned())),
    };

    // Max silence time is 4 weeks
    if silence_seconds > 0x24EA00 {
        return Ok(Some("Invalid silence time. Max silence time is 4 weeks.".to_owned()));
    }

    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Silence the user
    users::silence_user(ctx, target_user.user_id, &args.reason, silence_seconds).await?;

    // TODO: Log the action

    Ok(Some(format!("{} has been silenced for: {}.", args.username, args.reason)))
}

#[command(
    "unsilence",
    required_privileges = Privileges::AdminSilenceUsers,
)]
pub async fn unsilence_user<C: Context>(ctx: &C, sender: &Session, args: UnsilenceArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Unsilence the user
    users::silence_user(ctx, target_user.user_id, "", 0).await?;

    // TODO: Log the action

    Ok(Some(format!("{}'s silence reset.", args.username)))
}
