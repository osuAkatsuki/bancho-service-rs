use std::str::FromStr;

use crate::commands;
use crate::commands::{CommandResult, CommandRouterFactory};
use crate::common::context::Context;
use crate::common::error::AppError;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{multiplayer, sessions, streams, users};
use bancho_protocol::messages::server::ChatMessage;
use bancho_protocol::structures::{IrcMessage, Mods};
use bancho_service_macros::{FromCommandArgs, command};

pub static COMMANDS: CommandRouterFactory = commands![
    host,
    addref,
    rmref,
    listref,
    clearhost,
    lock,
    unlock,
    size,
    move_cmd,
    make,
    close,
    start,
    abort,
    invite_user,
    map,
    set,
    kick,
    password,
    randompassword,
    change_mods,
    team,
    settings,
    scorev,
    help,
    link,
    timer,
    aborttimer,
];

#[derive(Debug, FromCommandArgs)]
pub struct HostArgs {
    pub safe_username: String,
}

#[command("host")]
pub async fn host<C: Context>(ctx: &C, sender: &Session, args: HostArgs) -> CommandResult {
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
pub async fn addref<C: Context>(ctx: &C, sender: &Session, args: AddRefereeArgs) -> CommandResult {
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
pub async fn rmref<C: Context>(
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
pub async fn listref<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
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
pub async fn clearhost<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
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

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    // TODO: Need update_match function to set is_locked=true
    // await match.update_match(mp_match.match_id, is_locked=true);

    Ok(Some("This match has been locked.".to_string()))
}

#[command("unlock")]
pub async fn unlock<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    // TODO: Need update_match function to set is_locked=false
    // await match.update_match(mp_match.match_id, is_locked=false);

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

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    // TODO: Need force_size function
    // await match.forceSize(mp_match.match_id, args.match_size);

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
    pub time: u32,
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

    // TODO: Need start_match function and timer logic
    // if let Some(start_time) = args.time {
    //     // Start countdown timer
    //     return Ok(Some(format!("Match starts in {} seconds. The match has been locked.", start_time)));
    // } else {
    //     // Start immediately
    //     let success = await match.start(mp_match.match_id);
    //     if success {
    //         return Ok(Some("Starting match".to_string()));
    //     } else {
    //         return Ok(Some("Couldn't start match. Make sure there are enough players and teams are valid.".to_string()));
    //     }
    // }

    Ok(Some("Starting match".to_string()))
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

    // TODO: multiplayer::abort(ctx, mp_match.match_id).await?;
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
pub async fn map<C: Context>(ctx: &C, sender: &Session, args: MapArgs) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    if let Some(gamemode) = args.gamemode {
        if gamemode > 3 {
            return Ok(Some("Gamemode must be 0, 1, 2 or 3.".to_string()));
        }
    }

    // TODO: Need update_match function to change beatmap and gamemode
    // await match.update_match(mp_match.match_id, beatmap_id=args.beatmap_id, game_mode=gamemode.unwrap_or(0));

    Ok(Some("Match map has been updated.".to_string()))
}

#[derive(Debug, FromCommandArgs)]
pub struct SetArgs {
    pub team_mode: u8,
    pub score_mode: Option<u8>,
    pub match_size: Option<u8>,
}

#[command("set")]
pub async fn set<C: Context>(ctx: &C, sender: &Session, args: SetArgs) -> CommandResult {
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

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    // TODO: Need update_match function to change team_type, scoring_type, and size
    // await match.update_match(mp_match.match_id, match_team_type=args.team_mode, match_scoring_type=args.score_mode.unwrap_or(mp_match.match_scoring_type));

    Ok(Some("Match settings have been updated!".to_string()))
}

#[derive(Debug, FromCommandArgs)]
pub struct KickArgs {
    pub safe_username: String,
}

#[command("kick")]
pub async fn kick<C: Context>(ctx: &C, sender: &Session, args: KickArgs) -> CommandResult {
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
    pub new_password: Option<String>,
}

#[command("password", forward_message = false)]
pub async fn password<C: Context>(ctx: &C, sender: &Session, args: PasswordArgs) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    let _new_password = args.new_password.unwrap_or_else(|| "".to_string());

    // TODO: Need change_password function
    // await match.changePassword(mp_match.match_id, new_password);

    Ok(Some("Match password has been changed!".to_string()))
}

#[command("randompassword")]
pub async fn randompassword<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    // TODO: Need change_password function with random password
    // let new_password = Uuid::new_v4().to_string();
    // multiplayer::change_password(ctx, match_id, &new_password).await?;

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
pub struct TeamArgs {
    pub safe_username: String,
    pub colour: String,
}

#[command("team")]
pub async fn team<C: Context>(ctx: &C, sender: &Session, args: TeamArgs) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    // TODO: Check if team vs mode
    // if mp_match.match_team_type != matchTeamTypes.TEAM_VS && mp_match.match_team_type != matchTeamTypes.TAG_TEAM_VS {
    //     return Ok(Some("Command only available in team vs.".to_string()));
    // }

    let colour = args.colour.to_lowercase();
    if colour != "red" && colour != "blue" {
        return Ok(Some("Team colour must be red or blue.".to_string()));
    }

    let target_user = users::fetch_one_by_username_safe(ctx, &args.safe_username).await?;

    // TODO: Need change_team function
    // await match.changeTeam(mp_match.match_id, target_user.user_id, colour_const);

    Ok(Some(format!(
        "{} is now in {} team",
        target_user.username, colour
    )))
}

#[command("settings")]
pub async fn settings<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    // TODO: Need get_slots function and slot status formatting
    // let slots = await slot.get_slots(mp_match.match_id);
    // let mut msg = vec!["PLAYERS IN THIS MATCH ".to_string()];
    // if !args.single_line {
    //     msg.push("(use !mp settings single for a single-line version):\n".to_string());
    // } else {
    //     msg.push(": ".to_string());
    // }

    Ok(Some(
        "PLAYERS IN THIS MATCH: TODO - need slot functions".to_string(),
    ))
}

#[derive(Debug, FromCommandArgs)]
pub struct ScoreVArgs {
    pub version: String,
}

#[command("scorev")]
pub async fn scorev<C: Context>(ctx: &C, sender: &Session, args: ScoreVArgs) -> CommandResult {
    if args.version != "1" && args.version != "2" {
        return Ok(Some("Incorrect syntax: !mp scorev <1|2>.".to_string()));
    }

    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    // TODO: Need update_match function to change scoring type
    // let new_scoring_type = if args.version == "2" { matchScoringTypes.SCORE_V2 } else { matchScoringTypes.SCORE };
    // await match.update_match(mp_match.match_id, match_scoring_type=new_scoring_type);

    Ok(Some(format!(
        "Match scoring type set to scorev{}.",
        args.version
    )))
}

#[command("help")]
pub async fn help<C: Context>(_ctx: &C, _sender: &Session) -> CommandResult {
    Ok(Some(format!("Supported multiplayer subcommands: <>.")))
}

#[command("link")]
pub async fn link<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Need get_match_history_message function
    // return match.get_match_history_message(mp_match.match_id, mp_match.match_history_private);

    Ok(Some(
        "Match history link: TODO - need match history function".to_string(),
    ))
}

#[derive(Debug, FromCommandArgs)]
pub struct TimerArgs {
    pub time: u32,
}

#[command("timer")]
pub async fn timer<C: Context>(ctx: &C, sender: &Session, args: TimerArgs) -> CommandResult {
    if args.time < 1 {
        return Ok(Some(
            "Countdown time must be at least 1 second.".to_string(),
        ));
    }

    if args.time > 300 {
        return Ok(Some(
            "Countdown time must be less than 5 minutes.".to_string(),
        ));
    }

    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Check if user is referee - need referee functions
    // let referees = match::get_referees(mp_match.match_id).await?;
    // if !referees.contains(&sender.user_id) {
    //     return Ok(Some("You are not a referee for this match.".to_string()));
    // }

    // TODO: Need timer functions and countdown logic
    // if mp_match.is_timer_running {
    //     return Ok(Some("A countdown is already running.".to_string()));
    // }

    // await match.update_match(mp_match.match_id, is_timer_running=true);

    let minutes = args.time / 60;
    let seconds = args.time % 60;
    let message = if minutes > 0 && seconds == 0 {
        format!("Countdown ends in {} minute(s)", minutes)
    } else {
        format!("Countdown ends in {} second(s)", seconds)
    };

    Ok(Some(message))
}

#[command("aborttimer")]
pub async fn aborttimer<C: Context>(ctx: &C, sender: &Session) -> CommandResult {
    let match_id = multiplayer::fetch_session_match_id(ctx, sender.session_id)
        .await?
        .ok_or(AppError::MultiplayerUserNotInMatch)?;

    let _mp_match = multiplayer::fetch_one(ctx, match_id).await?;

    // TODO: Need update_match function to set is_timer_running=false
    // await match.update_match(mp_match.match_id, is_timer_running=false);

    Ok(Some("Countdown stopped.".to_string()))
}
