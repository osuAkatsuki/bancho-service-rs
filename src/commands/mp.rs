use std::str::FromStr;

use crate::commands;
use crate::commands::{CommandResult, CommandRouterFactory};
use crate::common::context::Context;
use crate::common::error::AppError;
use crate::common::website;
use crate::entities::gamemodes::Gamemode;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::multiplayer::TimerType;
use crate::repositories::streams::StreamName;
use crate::usecases::{beatmaps, multiplayer, sessions, streams, users};
use bancho_protocol::messages::server::ChatMessage;
use bancho_protocol::structures::{IrcMessage, MatchTeam, MatchTeamType, Mode, Mods, WinCondition};
use bancho_service_macros::{FromCommandArgs, command};

pub static COMMANDS: CommandRouterFactory = commands![
    set_host,
    add_referee,
    remove_referee,
    list_referees,
    clear_host,
    lock,
    unlock,
    size,
    move_cmd,
    make,
    close,
    start,
    abort,
    invite_user,
    set_map,
    set_settings,
    kick_user,
    set_password,
    randomize_password,
    change_mods,
    set_user_team,
    view_settings,
    set_scorev,
    help,
    match_history_link,
    timer,
    aborttimer,
];

#[derive(Debug, FromCommandArgs)]
pub struct SetHostArgs {
    pub safe_username: String,
}

#[command("host")]
pub async fn set_host<C: Context>(ctx: &C, sender: &Session, args: SetHostArgs) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;
    multiplayer::transfer_host_to_user(ctx, match_id, target_user.user_id, Some(sender.user_id))
        .await?;
    Ok(Some(format!("{} is now the host", target_user.username)))
}

#[derive(Debug, FromCommandArgs)]
pub struct AddRefereeArgs {
    pub safe_username: String,
}

#[command("addref")]
pub async fn add_referee<C: Context>(
    ctx: &C,
    sender: &Session,
    args: AddRefereeArgs,
) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;
    multiplayer::add_referee(ctx, match_id, target_user.user_id, Some(sender.user_id)).await?;
    Ok(Some(format!("Added {} to referees", target_user.username)))
}

#[derive(Debug, FromCommandArgs)]
pub struct RemoveRefereeArgs {
    pub safe_username: String,
}

#[command("rmref")]
pub async fn remove_referee<C: Context>(
    ctx: &C,
    sender: &Session,
    args: RemoveRefereeArgs,
) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;
    multiplayer::remove_referee(ctx, match_id, target_user.user_id, Some(sender.user_id)).await?;
    Ok(Some(format!(
        "Removed {} from referees",
        target_user.username
    )))
}

#[command("listref")]
pub async fn list_referees<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    let referees = multiplayer::get_referees(ctx, match_id).await?;
    let mut ref_usernames: Vec<String> = vec![];
    for ref_id in referees {
        let referee = users::fetch_one(ctx, ref_id).await?;
        ref_usernames.push(referee.username);
    }

    Ok(Some(format!(
        "Referees for this match: {}",
        ref_usernames.join(", ")
    )))
}

#[command("clearhost")]
pub async fn clear_host<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }
    multiplayer::clear_host(ctx, mp_match.match_id).await?;

    Ok(Some("Host has been removed from this match.".to_string()))
}

#[command("lock")]
pub async fn lock<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    multiplayer::lock_match(ctx, match_id).await?;

    Ok(Some("This match has been locked.".to_string()))
}

#[command("unlock")]
pub async fn unlock<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    multiplayer::unlock_match(ctx, match_id).await?;

    Ok(Some("This match has been unlocked.".to_string()))
}

#[derive(Debug, FromCommandArgs)]
pub struct SizeArgs {
    pub match_size: u8,
}

#[command("size")]
pub async fn size<C: Context>(ctx: &C, sender: &Session, args: SizeArgs) -> CommandResult {
    if args.match_size < 2 || args.match_size > 16 {
        return Ok(Some("Match size must be between 2 and 16.".to_string()));
    }

    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    multiplayer::resize_match(ctx, match_id, args.match_size as _).await?;
    Ok(Some(format!("Match size changed to {}.", args.match_size)))
}

#[derive(Debug, FromCommandArgs)]
pub struct MoveArgs {
    pub safe_username: String,
    pub slot: u8,
}

#[command("move")]
pub async fn move_cmd<C: Context>(ctx: &C, sender: &Session, args: MoveArgs) -> CommandResult {
    if args.slot > 16 {
        return Ok(Some("Slot ID must be between 0 and 16.".to_string()));
    }

    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;
    let (slot_id, _slot) = multiplayer::fetch_user_slot(ctx, match_id, target_user.user_id).await?;

    multiplayer::swap_slots(ctx, match_id, slot_id, args.slot as _).await?;

    Ok(Some(format!(
        "{} moved to slot {}.",
        target_user.username, args.slot
    )))
}

#[derive(Debug, FromCommandArgs)]
pub struct MakeArgs {
    pub name: String,
}

#[command(
    "make",
    required_privileges = Privileges::AdminTournamentStaff,
)]
pub async fn make<C: Context>(ctx: &C, sender: &Session, args: MakeArgs) -> CommandResult {
    if args.name.trim().is_empty() {
        return Ok(Some("Match name must not be empty!".to_string()));
    }

    // Check if user is already in a match
    if let Some(_) = multiplayer::fetch_session_match_id(ctx, sender.session_id).await? {
        return Ok(Some("You are already in a match.".to_string()));
    }

    let mp_match = multiplayer::create(
        ctx,
        sender,
        &args.name,
        "",
        "Tournament",
        "",
        0,
        crate::entities::gamemodes::Gamemode::Standard,
        16,
    )
    .await?;

    Ok(Some(format!(
        "Tourney match created with ID {}.",
        mp_match.match_id
    )))
}

#[command("close")]
pub async fn close<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    let slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    for slot in slots {
        match slot.user {
            Some(slot_user) => {
                multiplayer::leave(ctx, slot_user, Some(match_id)).await?;
            }
            None => {}
        }
    }

    multiplayer::delete(ctx, mp_match.match_id).await?;
    Ok(Some(format!(
        "Multiplayer match #{} disposed successfully.",
        mp_match.match_id
    )))
}

#[derive(Debug, FromCommandArgs)]
pub struct StartArgs {
    pub timer_seconds: u32,
}

#[command("start")]
pub async fn start<C: Context>(
    ctx: &C,
    sender: &Session,
    args: Option<StartArgs>,
) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    match args {
        Some(args) => {
            multiplayer::start_timer(
                ctx,
                match_id,
                TimerType::MatchStart,
                args.timer_seconds as u64,
            );
            Ok(Some(format!(
                "Countdown started. Match starts in {} second(s).",
                args.timer_seconds,
            )))
        }
        None => {
            multiplayer::start_game(ctx, match_id, None).await?;
            Ok(Some("Starting match".to_string()))
        }
    }
}

#[command("abort")]
pub async fn abort<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    if mp_match.in_progress {
        multiplayer::abort(ctx, mp_match.match_id).await?;
    } else {
        multiplayer::abort_timer(ctx, match_id, TimerType::MatchStart).await?;
    }

    Ok(Some("Match aborted!".to_string()))
}

#[derive(Debug, FromCommandArgs)]
pub struct InviteArgs {
    pub safe_username: String,
}

#[command("invite")]
pub async fn invite_user<C: Context>(ctx: &C, sender: &Session, args: InviteArgs) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    let target_session = sessions::fetch_primary_by_username(ctx, &args.safe_username).await?;
    let invite = mp_match.invite_message();
    let invite_message = IrcMessage {
        sender: &sender.username,
        sender_id: sender.user_id as _,
        text: &invite,
        recipient: &target_session.username,
    };
    streams::broadcast_message(
        ctx,
        StreamName::User(target_session.session_id),
        ChatMessage(&invite_message),
        None,
        None,
    )
    .await?;

    Ok(Some(format!(
        "An invite to this match has been sent to {}.",
        target_session.username
    )))
}

#[derive(Debug, FromCommandArgs)]
pub struct MapArgs {
    pub beatmap_id: i32,
    pub gamemode: Option<u8>,
}

#[command("map")]
pub async fn set_map<C: Context>(ctx: &C, sender: &Session, args: MapArgs) -> CommandResult {
    if let Some(gamemode) = args.gamemode {
        if gamemode > 3 {
            return Ok(Some("Gamemode must be 0, 1, 2 or 3.".to_string()));
        }
    }

    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mut mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    // Fetch the beatmap to get its details
    let beatmap = beatmaps::fetch_by_id(ctx, args.beatmap_id).await?;
    mp_match.beatmap_id = beatmap.beatmap_id;
    mp_match.beatmap_name = beatmap.song_name;
    mp_match.beatmap_md5 = beatmap.beatmap_md5;

    let new_mode = match args.gamemode {
        Some(gamemode) if beatmap.mode == 0 => gamemode,
        _ => beatmap.mode as u8,
    };
    let new_mode = Mode::try_from(new_mode)
        .map_err(|_| AppError::CommandsInvalidArgument("Invalid gamemode"))?;

    // osu! mode changed, reset mods.
    if mp_match.mode.as_bancho() != new_mode {
        mp_match.mods = Mods::None;
        let mut slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
        for slot in &mut slots {
            slot.mods = Mods::None;
        }
        multiplayer::update_all_slots(ctx, match_id, slots).await?;
    }

    mp_match.mode = Gamemode::from_mode_and_mods(new_mode, mp_match.mods);
    multiplayer::update(ctx, mp_match).await?;

    Ok(Some("Match map has been updated.".to_string()))
}

#[derive(Debug, FromCommandArgs)]
pub struct SetArgs {
    pub team_mode: u8,
    pub score_mode: Option<u8>,
    pub match_size: Option<u8>,
}

#[command("set")]
pub async fn set_settings<C: Context>(ctx: &C, sender: &Session, args: SetArgs) -> CommandResult {
    if args.team_mode > 3 {
        return Ok(Some("Match team type must be between 0 and 3.".to_string()));
    }

    if let Some(score_mode) = args.score_mode {
        if score_mode > 3 {
            return Ok(Some(
                "Match scoring type must be between 0 and 3.".to_string(),
            ));
        }
    }

    if let Some(match_size) = args.match_size {
        if match_size < 2 || match_size > 16 {
            return Ok(Some("Match size must be between 2 and 16.".to_string()));
        }
    }

    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mut mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    // Update match settings
    mp_match.team_type = MatchTeamType::try_from(args.team_mode)
        .map_err(|_| AppError::CommandsInvalidArgument("Invalid team mode"))?;
    if let Some(score_mode) = args.score_mode {
        mp_match.win_condition = WinCondition::try_from(score_mode)
            .map_err(|_| AppError::CommandsInvalidArgument("Invalid score mode"))?;
    }

    // Update the match
    multiplayer::update(ctx, mp_match).await?;

    // Update match size if argument is present
    if let Some(match_size) = args.match_size {
        multiplayer::resize_match(ctx, match_id, match_size as _).await?;
    }

    Ok(Some("Match settings have been updated!".to_string()))
}

#[derive(Debug, FromCommandArgs)]
pub struct KickArgs {
    pub safe_username: String,
}

#[command("kick")]
pub async fn kick_user<C: Context>(ctx: &C, sender: &Session, args: KickArgs) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    let slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    let user_slots = slots.iter().filter(|slot| {
        slot.user
            .is_some_and(|identity| identity.user_id == target_user.user_id)
    });

    for slot in user_slots {
        multiplayer::leave(ctx, slot.user.unwrap(), Some(match_id)).await?;
    }

    Ok(Some(format!(
        "{} has been kicked from the match.",
        target_user.username
    )))
}

#[derive(Debug, FromCommandArgs)]
pub struct PasswordArgs {
    pub new_password: String,
}

#[command("password", forward_message = false)]
pub async fn set_password<C: Context>(
    ctx: &C,
    sender: &Session,
    args: PasswordArgs,
) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mut mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    // Update match password
    mp_match.password = args.new_password;
    multiplayer::update(ctx, mp_match).await?;

    Ok(Some("Match password has been changed!".to_string()))
}

#[command("randompassword")]
pub async fn randomize_password<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mut mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    // Generate random password
    let new_password = uuid::Uuid::new_v4().to_string();
    mp_match.password = new_password;
    crate::repositories::multiplayer::update(ctx, mp_match.as_entity(), true).await?;

    Ok(Some("Match password has been randomized.".to_string()))
}

#[derive(Debug, FromCommandArgs)]
pub struct ModsArgs {
    pub mod_string: String,
}

#[command("mods")]
pub async fn change_mods<C: Context>(ctx: &C, sender: &Session, args: ModsArgs) -> CommandResult {
    let mods = Mods::from_str(&args.mod_string).map_err(|_| {
        AppError::CommandsInvalidArgument("Invalid Mods. Correct syntax: e.g. HDHR, EZDTFL, etc.")
    })?;

    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;
    multiplayer::change_mods(ctx, match_id, mods, Some(sender.identity())).await?;
    Ok(Some("Match mods have been updated!".to_string()))
}

#[derive(Debug, FromCommandArgs)]
pub struct SetUserTeamArgs {
    pub safe_username: String,
    pub colour: String,
}

#[command("team")]
pub async fn set_user_team<C: Context>(
    ctx: &C,
    sender: &Session,
    args: SetUserTeamArgs,
) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    if mp_match.team_type != MatchTeamType::Vs && mp_match.team_type != MatchTeamType::TagVs {
        return Ok(Some("Command only available in versus mode.".to_string()));
    }

    let colour = args.colour.to_lowercase();
    if colour != "red" && colour != "blue" {
        return Ok(Some("Team colour must be red or blue.".to_string()));
    }

    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    let new_team = if colour == "red" {
        MatchTeam::Red
    } else {
        MatchTeam::Blue
    };
    multiplayer::set_user_team(ctx, match_id, target_user.user_id, new_team).await?;

    Ok(Some(format!(
        "{} is now in {} team",
        target_user.username, colour
    )))
}

#[command("settings")]
pub async fn view_settings<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    let slots = multiplayer::fetch_all_slots(ctx, match_id).await?;
    let mut msg = vec!["PLAYERS IN THIS MATCH".to_string()];
    for slot in slots.iter().filter_map(|slot| slot.user) {
        let user = users::fetch_one(ctx, slot.user_id).await?;
        msg.push(format!("{} ({})", user.username, user.user_id));
    }

    Ok(Some(msg.join("\n")))
}

#[derive(Debug, FromCommandArgs)]
pub struct SetScoreVArgs {
    pub version: String,
}

#[command("scorev")]
pub async fn set_scorev<C: Context>(
    ctx: &C,
    sender: &Session,
    args: SetScoreVArgs,
) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mut mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    // Update scoring type
    let new_scoring_type = if args.version == "2" {
        WinCondition::ScoreV2
    } else {
        WinCondition::Score
    };
    mp_match.win_condition = new_scoring_type;
    multiplayer::update(ctx, mp_match).await?;

    Ok(Some(format!(
        "Match win condition set to {:?}.",
        new_scoring_type,
    )))
}

#[command("help")]
pub async fn help<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(Some(format!("Supported multiplayer subcommands: <>.")))
}

#[command("link")]
pub async fn match_history_link<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_history_link = website::get_match_history_link(match_id);
    let message = format!("Match history available [{mp_history_link} here].");
    Ok(Some(message))
}

#[derive(Debug, FromCommandArgs)]
pub struct TimerArgs {
    pub seconds: u32,
}

#[command("timer")]
pub async fn timer<C: Context>(ctx: &C, sender: &Session, args: TimerArgs) -> CommandResult {
    if args.seconds < 1 {
        return Ok(Some(
            "Countdown time must be at least 1 second.".to_string(),
        ));
    }

    if args.seconds > 300 {
        return Ok(Some(
            "Countdown time must be less than 5 minutes.".to_string(),
        ));
    }

    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    multiplayer::start_timer(ctx, match_id, TimerType::Regular, args.seconds as u64);
    Ok(Some(format!(
        "Countdown started. Ends in {} second(s).",
        args.seconds,
    )))
}

#[command("aborttimer")]
pub async fn aborttimer<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let mp_match = multiplayer::fetch_one(ctx, match_id).await?;
    if mp_match.host_user_id != sender.user_id
        && !multiplayer::is_referee(ctx, match_id, sender.user_id).await?
    {
        return Err(AppError::MultiplayerUnauthorized);
    }

    multiplayer::abort_timer(ctx, match_id, TimerType::Regular).await?;

    Ok(Some("Countdown stopped.".to_string()))
}
