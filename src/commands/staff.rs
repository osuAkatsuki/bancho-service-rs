use crate::adapters::discord;
use crate::commands::CommandResult;
use crate::common::context::Context;
use crate::common::website;
use crate::entities::bot;
use crate::models::bancho::LoginError;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{badges, sessions, streams, tillerino, users};
use bancho_protocol::messages::server::{ChatMessage, LoginResult};
use bancho_protocol::structures::IrcMessage;
use bancho_service_macros::{FromCommandArgs, command};

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
    let last_np = tillerino::fetch_last_np(ctx, sender.session_id).await?;
    if last_np.is_none() {
        return Ok(Some("Please /np a map first!".to_owned()));
    }

    // TODO: Map ranking/unranking logic would require integration with the beatmap ranking system
    // This is a complex feature that needs dedicated beatmap management infrastructure
    Ok(Some(
        "Map editing functionality not yet implemented.".to_owned(),
    ))
}

#[derive(Debug, FromCommandArgs)]
pub struct AddBNArgs {
    pub username: String,
}

#[command(
    "addbn",
    required_privileges = Privileges::AdminManageNominators,
)]
pub async fn add_bn<C: Context>(ctx: &C, sender: &Session, args: AddBNArgs) -> CommandResult {
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

    // Add BN badge
    badges::add_user_badge(ctx, target_user.user_id, "Beatmap Nominator").await?;

    // Set donor expiry to permanent (2147483647 = max i32)
    users::update_donor_expiry(ctx, target_user.user_id, 2147483647).await?;

    let log_message = format!("{} has given BN to {}.", sender.username, args.username);
    let _ = discord::send_blue_embed("BN Added", &log_message, None).await;

    Ok(Some(format!(
        "{} has given BN to {}.",
        sender.username, args.username
    )))
}

#[derive(Debug, FromCommandArgs)]
pub struct RemoveBNArgs {
    pub username: String,
}

#[command(
    "removebn",
    required_privileges = Privileges::AdminManageNominators,
)]
pub async fn remove_bn<C: Context>(ctx: &C, sender: &Session, args: RemoveBNArgs) -> CommandResult {
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

    // Remove BN badge
    badges::remove_user_badge(ctx, target_user.user_id, "Beatmap Nominator").await?;

    // Set donor expiry to 0 (expired)
    users::update_donor_expiry(ctx, target_user.user_id, 0).await?;

    let log_message = format!("has removed BN from {}", args.username);
    let _ = discord::send_blue_embed("BN Removed", &log_message, None).await;

    Ok(Some(format!(
        "{} has removed BN from {}.",
        sender.username, args.username
    )))
}

#[command(
    "kick",
    required_privileges = Privileges::AdminKickUsers,
)]
pub async fn kick<C: Context>(ctx: &C, sender: &Session, args: KickArgs) -> CommandResult {
    let target_sessions = sessions::fetch_by_username(ctx, &args.username).await?;

    let mut session_count = 0;
    for session in target_sessions {
        sessions::delete(ctx, &session).await?;
        session_count += 1;
    }

    if session_count == 0 {
        return Ok(Some("Target not online.".to_owned()));
    }

    let log_message = format!("has kicked {} for: {}", args.username, args.reason);
    let _ = discord::send_red_embed("User Kicked", &log_message, None).await;

    Ok(Some(format!(
        "{} has been kicked from the server.",
        args.username
    )))
}

#[command(
    "ban",
    required_privileges = Privileges::AdminManageBans,
)]
pub async fn ban_user<C: Context>(ctx: &C, sender: &Session, args: BanArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Ban the user (remove login privileges)
    users::ban_user(ctx, target_user.user_id).await?;

    // Send ban packet to online sessions
    let target_sessions = sessions::fetch_by_username(ctx, &args.username).await?;
    for session in target_sessions {
        let ban_packet = LoginResult {
            user_id: LoginError::Banned as i32,
        };
        streams::broadcast_message(
            ctx,
            StreamName::User(session.session_id),
            ban_packet,
            None,
            None,
        )
        .await?;
    }

    let log_message = format!("has banned {} for: {}", args.username, args.reason);
    let _ = discord::send_red_embed("User Banned", &log_message, None).await;

    Ok(Some(format!("{} has been banned.", args.username)))
}

#[command(
    "unban",
    required_privileges = Privileges::AdminManageBans,
)]
pub async fn unban_user<C: Context>(ctx: &C, sender: &Session, args: UnbanArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;
    users::unban_user(ctx, target_user.user_id).await?;

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has unbanned [{}]({}) for: {}",
        sender.username, sender_profile, args.username, target_profile, args.reason
    );
    let _ = discord::send_blue_embed("User Unbanned", &log_message, None).await;

    let osu_format_reply = format!("[{} {}] has been unbanned", target_profile, args.username);
    Ok(Some(osu_format_reply))
}

#[command(
    "restrict",
    required_privileges = Privileges::AdminManageBans,
)]
pub async fn restrict_user<C: Context>(
    ctx: &C,
    sender: &Session,
    args: RestrictArgs,
) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Restrict the user (remove publicly visible privileges)
    users::restrict_user(ctx, target_user.user_id).await?;

    // Notify online sessions
    let target_sessions = sessions::fetch_by_username(ctx, &args.username).await?;
    for session in target_sessions {
        // Send restriction notification packet
        let restriction_message = IrcMessage {
            recipient: &args.username,
            sender: bot::BOT_NAME,
            sender_id: bot::BOT_ID as _,
            text: "Your account is now in restricted mode. Visit the website for more information.",
        };
        streams::broadcast_message(
            ctx,
            StreamName::User(session.session_id),
            ChatMessage(&restriction_message),
            None,
            None,
        )
        .await?;
    }

    let log_message = format!(
        "{} has restricted {} for: {}",
        sender.username, args.username, args.reason
    );
    let _ = discord::send_red_embed("User Restricted", &log_message, None).await;
    Ok(Some(log_message))
}

#[command(
    "unrestrict",
    required_privileges = Privileges::AdminManageBans,
)]
pub async fn unrestrict_user<C: Context>(
    ctx: &C,
    sender: &Session,
    args: UnrestrictArgs,
) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Unrestrict the user (restore publicly visible privileges)
    users::unrestrict_user(ctx, target_user.user_id).await?;

    let log_message = format!("has unrestricted {} for: {}", args.username, args.reason);
    let _ = discord::send_blue_embed("User Unrestricted", &log_message, None).await;

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

    let log_message = format!("has frozen {} for: {}", args.username, args.reason);
    let _ = discord::send_red_embed("User Frozen", &log_message, None).await;

    Ok(Some(format!("Froze {}.", args.username)))
}

#[command(
    "unfreeze",
    required_privileges = Privileges::AdminFreezeUsers,
)]
pub async fn unfreeze_user<C: Context>(
    ctx: &C,
    sender: &Session,
    args: UnfreezeArgs,
) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Check if user is frozen
    if !target_user.frozen {
        return Ok(Some("That user is not frozen.".to_owned()));
    }

    // Unfreeze the user
    users::unfreeze_user(ctx, target_user.user_id).await?;

    let log_message = format!("has unfrozen {} for: {}", args.username, args.reason);
    let _ = discord::send_blue_embed("User Unfrozen", &log_message, None).await;

    Ok(Some(format!("Unfroze {}.", args.username)))
}

#[command(
    "whitelist",
    required_privileges = Privileges::AdminManageUsers,
)]
pub async fn whitelist_user<C: Context>(
    ctx: &C,
    sender: &Session,
    args: WhitelistArgs,
) -> CommandResult {
    if !(0..4).contains(&args.bit) {
        return Ok(Some("Invalid bit.".to_owned()));
    }

    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Update whitelist status
    users::update_user_whitelist(ctx, target_user.user_id, args.bit).await?;

    let log_message = format!(
        "has set {}'s whitelist status to {} for: {}",
        args.username, args.bit, args.reason
    );
    let _ = discord::send_blue_embed("Whitelist Updated", &log_message, None).await;

    Ok(Some(format!(
        "{}'s Whitelist Status has been set to {}.",
        args.username, args.bit
    )))
}

#[command(
    "silence",
    required_privileges = Privileges::AdminSilenceUsers,
)]
pub async fn silence_user<C: Context>(
    ctx: &C,
    sender: &Session,
    args: SilenceArgs,
) -> CommandResult {
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
        return Ok(Some(
            "Invalid silence time. Max silence time is 4 weeks.".to_owned(),
        ));
    }

    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Silence the user
    users::silence_user(
        ctx,
        target_user.user_id,
        &args.reason,
        silence_seconds as i64,
    )
    .await?;

    let log_message = format!("has silenced {} for: {}", args.username, args.reason);
    let _ = discord::send_red_embed("User Silenced", &log_message, None).await;

    Ok(Some(format!(
        "{} has been silenced for: {}.",
        args.username, args.reason
    )))
}

#[command(
    "unsilence",
    required_privileges = Privileges::AdminSilenceUsers,
)]
pub async fn unsilence_user<C: Context>(
    ctx: &C,
    sender: &Session,
    args: UnsilenceArgs,
) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // Unsilence the user
    users::silence_user(ctx, target_user.user_id, "", 0).await?;

    let log_message = format!("has unsilenced {} for: {}", args.username, args.reason);
    let _ = discord::send_blue_embed("User Unsilenced", &log_message, None).await;

    Ok(Some(format!("{}'s silence reset.", args.username)))
}
