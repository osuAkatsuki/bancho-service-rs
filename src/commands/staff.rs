use std::time::Duration;
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
pub struct EditMapArgs {
    pub action: String,
    pub scope: String,
}

#[command(
    "map",
    required_privileges = Privileges::AdminManageBeatmaps,
)]
pub async fn edit_map<C: Context>(ctx: &C, sender: &Session, args: EditMapArgs) -> CommandResult {
    const RANKED_STATUS_MODIFICATIONS: [&str; 3] = ["rank", "unrank", "love"];
    if !RANKED_STATUS_MODIFICATIONS.contains(&args.action.as_str()) {
        let reply = format!("Invalid action! Valid actions are: {RANKED_STATUS_MODIFICATIONS:?}");
        return Ok(Some(reply));
    }

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
    pub safe_username: String,
}

#[command(
    "addbn",
    required_privileges = Privileges::AdminManageNominators,
)]
pub async fn add_bn<C: Context>(ctx: &C, sender: &Session, args: AddBNArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // Add BN privileges
    let new_privileges = target_user.privileges
        | Privileges::AdminManageBeatmaps
        | Privileges::Donator
        | Privileges::AkatsukiPlus;

    users::update_user_privileges(ctx, target_user.user_id, new_privileges).await?;

    // Update all user sessions
    let target_sessions = sessions::fetch_by_username(ctx, &args.safe_username).await?;
    for session in target_sessions {
        let mut updated_session = session;
        updated_session.privileges = new_privileges;
        sessions::update(ctx, updated_session).await?;
    }

    // Add BN badge
    badges::add_user_badge(ctx, target_user.user_id, "Beatmap Nomination").await?;

    // Set donor expiry to permanent (2147483647 = max i32)
    users::update_donor_expiry(ctx, target_user.user_id, 2147483647).await?;

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has given BN to [{}]({}).",
        sender.username, sender_profile, target_user.username, target_profile
    );
    let _ = discord::send_purple_embed("BN Added", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}] has been given BN",
        target_profile, target_user.username
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct RemoveBNArgs {
    pub safe_username: String,
}

#[command(
    "removebn",
    required_privileges = Privileges::AdminManageNominators,
)]
pub async fn remove_bn<C: Context>(ctx: &C, sender: &Session, args: RemoveBNArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // Remove BN privileges
    let new_privileges = target_user.privileges
        & !Privileges::AdminManageBeatmaps
        & !Privileges::Donator
        & !Privileges::AkatsukiPlus;

    users::update_user_privileges(ctx, target_user.user_id, new_privileges).await?;

    // Update all user sessions
    let target_sessions = sessions::fetch_by_username(ctx, &target_user.username).await?;
    for session in target_sessions {
        let mut updated_session = session;
        updated_session.privileges = new_privileges;
        sessions::update(ctx, updated_session).await?;
    }

    // Remove BN badge
    badges::remove_user_badge(ctx, target_user.user_id, "Beatmap Nomination").await?;

    // Set donor expiry to 0 (expired)
    users::update_donor_expiry(ctx, target_user.user_id, 0).await?;

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has removed BN from [{}]({}).",
        sender.username, sender_profile, target_user.username, target_profile
    );
    let _ = discord::send_purple_embed("BN Removed", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}] has been removed from BN",
        target_profile, target_user.username
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct KickArgs {
    pub safe_username: String,
    pub reason: String,
}

#[command(
    "kick",
    required_privileges = Privileges::AdminKickUsers,
)]
pub async fn kick<C: Context>(ctx: &C, sender: &Session, args: KickArgs) -> CommandResult {
    let mut target_sessions = sessions::fetch_by_username(ctx, &args.safe_username)
        .await?
        .peekable();
    let target_user_id = match target_sessions.peek() {
        Some(session) => session.user_id,
        None => return Ok(Some("Target not online.".to_owned())),
    };

    for session in target_sessions {
        sessions::delete(ctx, &session).await?;
    }

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user_id);
    let log_message = format!(
        "[{}]({}) has kicked [{}]({}) for: {}",
        sender.username, sender_profile, args.safe_username, target_profile, args.reason
    );
    let _ = discord::send_purple_embed("User Kicked", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}] has been kicked from the server",
        target_profile, args.safe_username
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct BanArgs {
    pub safe_username: String,
    pub reason: String,
}

#[command(
    "ban",
    required_privileges = Privileges::AdminManageBans,
)]
pub async fn ban_user<C: Context>(ctx: &C, sender: &Session, args: BanArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // Ban the user (remove login privileges)
    users::ban_user(ctx, target_user.user_id).await?;

    // Send ban packet to online sessions
    let target_sessions = sessions::fetch_by_username(ctx, &target_user.username).await?;
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

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has banned [{}]({}) for: {}",
        sender.username, sender_profile, target_user.username, target_profile, args.reason
    );
    let _ = discord::send_red_embed("User Banned", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}] has been banned",
        target_profile, target_user.username
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct UnbanArgs {
    pub safe_username: String,
    pub reason: String,
}

#[command(
    "unban",
    required_privileges = Privileges::AdminManageBans,
)]
pub async fn unban_user<C: Context>(ctx: &C, sender: &Session, args: UnbanArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;
    users::unban_user(ctx, target_user.user_id).await?;

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has unbanned [{}]({}) for: {}",
        sender.username, sender_profile, target_user.username, target_profile, args.reason
    );
    let _ = discord::send_blue_embed("User Unbanned", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}] has been unbanned",
        target_profile, target_user.username
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct RestrictArgs {
    pub safe_username: String,
    pub reason: String,
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
    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // Restrict the user (remove publicly visible privileges)
    users::restrict_user(ctx, target_user.user_id).await?;

    // Notify online sessions
    let target_sessions = sessions::fetch_by_username(ctx, &target_user.username).await?;
    for session in target_sessions {
        // Send restriction notification packet
        let restriction_message = IrcMessage {
            recipient: &target_user.username,
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

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has restricted [{}]({}) for: {}",
        sender.username, sender_profile, target_user.username, target_profile, args.reason
    );
    let _ = discord::send_red_embed("User Restricted", &log_message, None).await;
    let osu_format_reply = format!(
        "[{} {}] has been restricted for: {}",
        target_profile, target_user.username, args.reason
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct UnrestrictArgs {
    pub safe_username: String,
    pub reason: String,
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
    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // Unrestrict the user (restore publicly visible privileges)
    users::unrestrict_user(ctx, target_user.user_id).await?;

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has unrestricted [{}]({}) for: {}",
        sender.username, sender_profile, target_user.username, target_profile, args.reason
    );
    let _ = discord::send_blue_embed("User Unrestricted", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}] has been unrestricted",
        target_profile, target_user.username
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct FreezeArgs {
    pub safe_username: String,
    pub reason: String,
}

#[command(
    "freeze",
    required_privileges = Privileges::AdminFreezeUsers,
)]
pub async fn freeze_user<C: Context>(ctx: &C, sender: &Session, args: FreezeArgs) -> CommandResult {
    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // Check if user is already frozen
    if target_user.frozen {
        return Ok(Some("That user is already frozen.".to_owned()));
    }

    // Freeze the user
    users::freeze_user(ctx, target_user.user_id, &args.reason).await?;

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has frozen [{}]({}) for: {}",
        sender.username, sender_profile, target_user.username, target_profile, args.reason
    );
    let _ = discord::send_red_embed("User Frozen", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}] has been frozen",
        target_profile, target_user.username
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct UnfreezeArgs {
    pub safe_username: String,
    pub reason: String,
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
    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // Check if user is frozen
    if !target_user.frozen {
        return Ok(Some("That user is not frozen.".to_owned()));
    }

    // Unfreeze the user
    users::unfreeze_user(ctx, target_user.user_id).await?;

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has unfrozen [{}]({}) for: {}",
        sender.username, sender_profile, target_user.username, target_profile, args.reason
    );
    let _ = discord::send_blue_embed("User Unfrozen", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}] has been unfrozen",
        target_profile, target_user.username
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct WhitelistArgs {
    pub safe_username: String,
    pub bit: i32,
    pub reason: String,
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
    if !(0..=3).contains(&args.bit) {
        return Ok(Some("Invalid bit.".to_owned()));
    }

    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // Update whitelist status
    users::update_user_whitelist(ctx, target_user.user_id, args.bit).await?;

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has set [{}]({})'s whitelist status to {} for: {}",
        sender.username,
        sender_profile,
        target_user.username,
        target_profile,
        args.bit,
        args.reason
    );
    let _ = discord::send_blue_embed("Whitelist Updated", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}]'s whitelist status has been set to {}",
        target_profile, target_user.username, args.bit
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct SilenceArgs {
    pub safe_username: String,
    pub duration: Duration,
    pub reason: String,
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
    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // Silence the user
    users::silence_user(
        ctx,
        target_user.user_id,
        &args.reason,
        args.duration.as_secs() as i64,
    )
    .await?;

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has silenced [{}]({}) for: {}",
        sender.username, sender_profile, target_user.username, target_profile, args.reason
    );
    let _ = discord::send_red_embed("User Silenced", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}] has been silenced for: {}",
        target_profile, target_user.username, args.reason
    );
    Ok(Some(osu_format_reply))
}

#[derive(Debug, FromCommandArgs)]
pub struct UnsilenceArgs {
    pub safe_username: String,
    pub reason: String,
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
    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // Unsilence the user
    users::silence_user(ctx, target_user.user_id, "", 0).await?;

    let sender_profile = website::get_profile_link(sender.user_id);
    let target_profile = website::get_profile_link(target_user.user_id);
    let log_message = format!(
        "[{}]({}) has unsilenced [{}]({}) for: {}",
        sender.username, sender_profile, target_user.username, target_profile, args.reason
    );
    let _ = discord::send_blue_embed("User Unsilenced", &log_message, None).await;

    let osu_format_reply = format!(
        "[{} {}]'s silence has been reset",
        target_profile, target_user.username
    );
    Ok(Some(osu_format_reply))
}
