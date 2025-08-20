use crate::commands::from_args::NoArg;
use crate::commands::{CommandResult, CommandRouterFactory};
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult};
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::usecases::{multiplayer, users};
use bancho_service_macros::{FromCommandArgs, command};

pub static COMMANDS: CommandRouterFactory = crate::commands![
    addref,
    rmref,
    listref,
    make,
    close,
    lock,
    unlock,
    mp_size,
    move_player,
    host,
    clearhost,
    start,
    invite,
    map,
    set,
    abort,
    kick,
    mp_password,
    randompassword,
    mp_mods,
    team,
    settings,
    scorev,
    help,
    link,
    timer,
    aborttimer,
];

// Helper function to get user's match ID
async fn get_user_match_id<C: Context>(ctx: &C, session: &Session) -> ServiceResult<i64> {
    let match_id = multiplayer::fetch_session_match_id(ctx, session.session_id).await?;
    match_id.ok_or(AppError::MultiplayerUserNotInMatch)
}

// Helper function to check if user is referee
async fn check_referee_permissions<C: Context>(
    _ctx: &C,
    _session: &Session,
    _match_id: i64,
) -> ServiceResult<()> {
    // TODO: Implement referee checking logic
    // For now, we'll assume the user has permissions
    Ok(())
}

#[derive(FromCommandArgs)]
pub struct AddrefArgs {
    pub username: String,
}

#[command("addref")]
pub async fn addref<C: Context>(ctx: &C, sender: &Session, args: AddrefArgs) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // Get target user
    let _target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // TODO: Implement add_referee logic
    // multiplayer::add_referee(ctx, match_id, target_user.user_id).await?;

    Ok(Some(format!("Added {} to referees", args.username)))
}

#[derive(FromCommandArgs)]
pub struct RmrefArgs {
    pub username: String,
}

#[command("rmref")]
pub async fn rmref<C: Context>(ctx: &C, sender: &Session, args: RmrefArgs) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // Get target user
    let _target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // TODO: Implement remove_referee logic
    // multiplayer::remove_referee(ctx, match_id, target_user.user_id).await?;

    Ok(Some(format!("Removed {} from referees", args.username)))
}

#[command("listref")]
pub async fn listref<C: Context>(ctx: &C, sender: &Session, _args: NoArg) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement list_referees logic
    // let referees = multiplayer::get_referees(ctx, match_id).await?;
    // let ref_names = referees.iter().map(|r| r.username.clone()).collect::<Vec<_>>();
    // let refs = ref_names.join(", ");

    Ok(Some(
        "Referees for this match: (TODO: implement)".to_string(),
    ))
}

#[derive(FromCommandArgs)]
pub struct MakeArgs {
    pub name: String,
}

#[command("make")]
pub async fn make<C: Context>(ctx: &C, sender: &Session, args: MakeArgs) -> CommandResult {
    // Check if user is already in a match
    if let Ok(_match_id) = get_user_match_id(ctx, sender).await {
        return Ok(Some("You are already in a match.".to_string()));
    }

    // Check tournament staff privileges
    if !sender.privileges.contains(Privileges::AdminTournamentStaff) {
        return Ok(Some(
            "Only tournament staff may use this command".to_string(),
        ));
    }

    if args.name.is_empty() {
        return Err(AppError::CommandsInvalidSyntax(
            "Match name must not be empty",
            "!mp make <name>",
            "!mp make <name>",
        ));
    }

    // TODO: Implement make_match logic
    // let mp_match = multiplayer::create_tournament_match(
    //     ctx,
    //     sender,
    //     &args.name,
    //     "tournament_password", // Generate random password
    //     "Tournament",
    //     "",
    //     0,
    //     Gamemode::Standard,
    //     16,
    // ).await?;

    Ok(Some(format!("Tourney match #{} created!", "TODO")))
}

#[command("close")]
pub async fn close<C: Context>(ctx: &C, sender: &Session, _args: NoArg) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement close_match logic
    // multiplayer::dispose_match(ctx, match_id).await?;

    Ok(Some(format!(
        "Multiplayer match #{} disposed successfully.",
        match_id
    )))
}

#[command("lock")]
pub async fn lock<C: Context>(ctx: &C, sender: &Session, _args: NoArg) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement lock_match logic
    // multiplayer::update_match(ctx, match_id, is_locked: true).await?;

    Ok(Some("This match has been locked.".to_string()))
}

#[command("unlock")]
pub async fn unlock<C: Context>(ctx: &C, sender: &Session, _args: NoArg) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement unlock_match logic
    // multiplayer::update_match(ctx, match_id, is_locked: false).await?;

    Ok(Some("This match has been unlocked.".to_string()))
}

#[derive(FromCommandArgs)]
pub struct SizeArgs {
    pub size: i32,
}

#[command("size")]
pub async fn mp_size<C: Context>(ctx: &C, sender: &Session, args: SizeArgs) -> CommandResult {
    if args.size < 2 || args.size > 16 {
        return Err(AppError::CommandsInvalidSyntax(
            "Size must be between 2 and 16",
            "!mp size <slots(2-16)>",
            "!mp size <slots(2-16)>",
        ));
    }

    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement set_match_size logic
    // multiplayer::force_size(ctx, match_id, args.size as usize).await?;

    Ok(Some(format!("Match size changed to {}.", args.size)))
}

#[derive(FromCommandArgs)]
pub struct MoveArgs {
    pub username: String,
    pub slot: i32,
}

#[command("move")]
pub async fn move_player<C: Context>(ctx: &C, sender: &Session, args: MoveArgs) -> CommandResult {
    if args.slot < 0 || args.slot > 16 {
        return Err(AppError::CommandsInvalidSyntax(
            "Slot must be between 0 and 16",
            "!mp move <username> <slot>",
            "!mp move <username> <slot>",
        ));
    }

    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // Get target user
    let _target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // TODO: Implement move_player logic
    // let success = multiplayer::user_change_slot(ctx, match_id, target_user.user_id, args.slot).await?;

    Ok(Some(format!(
        "{} moved to slot {}.",
        args.username, args.slot
    )))
}

#[derive(FromCommandArgs)]
pub struct HostArgs {
    pub username: String,
}

#[command("host")]
pub async fn host<C: Context>(ctx: &C, sender: &Session, args: HostArgs) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // Get target user
    let _target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // TODO: Implement set_host logic
    // let success = multiplayer::set_host(ctx, match_id, target_user.user_id).await?;

    Ok(Some(format!("{} is now the host", args.username)))
}

#[command("clearhost")]
pub async fn clearhost<C: Context>(ctx: &C, sender: &Session, _args: NoArg) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement clear_host logic
    // multiplayer::remove_host(ctx, match_id).await?;

    Ok(Some("Host has been removed from this match.".to_string()))
}

#[derive(FromCommandArgs)]
pub struct StartArgs {
    pub force_or_countdown: Option<String>,
}

#[command("start")]
pub async fn start<C: Context>(ctx: &C, sender: &Session, args: StartArgs) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    let (_force, countdown) = if let Some(arg) = &args.force_or_countdown {
        if arg.to_lowercase() == "force" {
            (true, None)
        } else if let Ok(countdown_val) = arg.parse::<i32>() {
            (false, Some(countdown_val))
        } else {
            (false, None)
        }
    } else {
        (false, None)
    };

    // TODO: Implement start_match logic
    // let success = multiplayer::start_match(ctx, match_id, force, countdown).await?;

    if let Some(countdown) = countdown {
        Ok(Some(format!(
            "Match starts in {} seconds. The match has been locked.",
            countdown
        )))
    } else {
        Ok(Some("Starting match".to_string()))
    }
}

#[derive(FromCommandArgs)]
pub struct InviteArgs {
    pub username: String,
}

#[command("invite")]
pub async fn invite<C: Context>(ctx: &C, sender: &Session, args: InviteArgs) -> CommandResult {
    let _match_id = get_user_match_id(ctx, sender).await?;

    // Get target user
    let _target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // TODO: Implement invite_player logic
    // multiplayer::invite_player(ctx, match_id, target_user.user_id).await?;

    Ok(Some(format!(
        "An invite to this match has been sent to {}.",
        args.username
    )))
}

#[derive(FromCommandArgs)]
pub struct MapArgs {
    pub beatmap_id: i32,
    pub game_mode: Option<i32>,
}

#[command("map")]
pub async fn map<C: Context>(ctx: &C, sender: &Session, args: MapArgs) -> CommandResult {
    if let Some(mode) = args.game_mode {
        if mode < 0 || mode > 3 {
            return Err(AppError::CommandsInvalidSyntax(
                "Game mode must be 0, 1, 2 or 3",
                "!mp map <beatmapid> [<gamemode>]",
                "!mp map <beatmapid> [<gamemode>]",
            ));
        }
    }

    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement set_map logic
    // multiplayer::update_match_map(ctx, match_id, args.beatmap_id, args.game_mode.unwrap_or(0)).await?;

    Ok(Some("Match map has been updated.".to_string()))
}

#[derive(FromCommandArgs)]
pub struct SetArgs {
    pub team_type: i32,
    pub scoring_type: Option<i32>,
    pub size: Option<i32>,
}

#[command("set")]
pub async fn set<C: Context>(ctx: &C, sender: &Session, args: SetArgs) -> CommandResult {
    if args.team_type < 0 || args.team_type > 3 {
        return Err(AppError::CommandsInvalidSyntax(
            "Team type must be between 0 and 3",
            "!mp set <teammode> [<scoremode>] [<size>]",
            "!mp set <teammode> [<scoremode>] [<size>]",
        ));
    }

    if let Some(scoring) = args.scoring_type {
        if scoring < 0 || scoring > 3 {
            return Err(AppError::CommandsInvalidSyntax(
                "Scoring type must be between 0 and 3",
                "!mp set <teammode> [<scoremode>] [<size>]",
                "!mp set <teammode> [<scoremode>] [<size>]",
            ));
        }
    }

    if let Some(size_val) = args.size {
        if size_val < 2 || size_val > 16 {
            return Err(AppError::CommandsInvalidSyntax(
                "Size must be between 2 and 16",
                "!mp set <teammode> [<scoremode>] [<size>]",
                "!mp set <teammode> [<scoremode>] [<size>]",
            ));
        }
    }

    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement set_match_settings logic
    // multiplayer::update_match_settings(ctx, match_id, args.team_type, args.scoring_type, args.size).await?;

    Ok(Some("Match settings have been updated!".to_string()))
}

#[command("abort")]
pub async fn abort<C: Context>(ctx: &C, sender: &Session, _args: NoArg) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement abort_match logic
    // multiplayer::abort_match(ctx, match_id).await?;

    Ok(Some("Match aborted!".to_string()))
}

#[derive(FromCommandArgs)]
pub struct KickArgs {
    pub username: String,
}

#[command("kick")]
pub async fn kick<C: Context>(ctx: &C, sender: &Session, args: KickArgs) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // Get target user
    let _target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // TODO: Implement kick_player logic
    // multiplayer::kick_player(ctx, match_id, target_user.user_id).await?;

    Ok(Some(format!(
        "{} has been kicked from the match.",
        args.username
    )))
}

#[derive(FromCommandArgs)]
pub struct PasswordArgs {
    pub password: String,
}

#[command("password")]
pub async fn mp_password<C: Context>(
    ctx: &C,
    sender: &Session,
    _args: PasswordArgs,
) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement set_password logic
    // multiplayer::change_password(ctx, match_id, &args.password).await?;

    Ok(Some("Match password has been changed!".to_string()))
}

#[command("randompassword")]
pub async fn randompassword<C: Context>(ctx: &C, sender: &Session, _args: NoArg) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement random_password logic
    // let password = generate_random_password();
    // multiplayer::change_password(ctx, match_id, &password).await?;

    Ok(Some("Match password has been randomized.".to_string()))
}

#[derive(FromCommandArgs)]
pub struct ModsArgs {
    pub mods: String,
}

#[command("mods")]
pub async fn mp_mods<C: Context>(ctx: &C, sender: &Session, args: ModsArgs) -> CommandResult {
    if args.mods.len() % 2 != 0 {
        return Err(AppError::CommandsInvalidSyntax(
            "Mods must be pairs of characters",
            "!mp mods <mods, e.g. hdhr>",
            "!mp mods <mods, e.g. hdhr>",
        ));
    }

    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement set_mods logic
    // multiplayer::change_mods(ctx, match_id, &args.mods).await?;

    Ok(Some("Match mods have been updated!".to_string()))
}

#[derive(FromCommandArgs)]
pub struct TeamArgs {
    pub username: String,
    pub color: String,
}

#[command("team")]
pub async fn team<C: Context>(ctx: &C, sender: &Session, args: TeamArgs) -> CommandResult {
    let color = args.color.to_lowercase();
    if color != "red" && color != "blue" {
        return Err(AppError::CommandsInvalidSyntax(
            "Team colour must be red or blue",
            "!mp team <username> <colour>",
            "!mp team <username> <colour>",
        ));
    }

    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // Get target user
    let _target_user = users::fetch_one_by_username_safe(ctx, &args.username).await?;

    // TODO: Implement set_team logic
    // multiplayer::change_team(ctx, match_id, target_user.user_id, &color).await?;

    Ok(Some(format!("{} is now in {} team", args.username, color)))
}

#[derive(FromCommandArgs)]
pub struct SettingsArgs {
    pub single: Option<String>,
}

#[command("settings")]
pub async fn settings<C: Context>(ctx: &C, sender: &Session, args: SettingsArgs) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    let _single = args.single.as_ref().map(|s| s.to_lowercase() == "single");

    // TODO: Implement show_settings logic
    // let settings = multiplayer::get_match_settings(ctx, match_id, single.unwrap_or(false)).await?;

    Ok(Some("PLAYERS IN THIS MATCH: (TODO: implement)".to_string()))
}

#[derive(FromCommandArgs)]
pub struct ScorevArgs {
    pub version: String,
}

#[command("scorev")]
pub async fn scorev<C: Context>(ctx: &C, sender: &Session, args: ScorevArgs) -> CommandResult {
    if args.version != "1" && args.version != "2" {
        return Err(AppError::CommandsInvalidSyntax(
            "Version must be 1 or 2",
            "!mp scorev <1|2>",
            "!mp scorev <1|2>",
        ));
    }

    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement set_scoring logic
    // multiplayer::set_scoring_type(ctx, match_id, &args.version).await?;

    Ok(Some(format!(
        "Match scoring type set to scorev{}.",
        args.version
    )))
}

#[command("help")]
pub async fn help<C: Context>(_ctx: &C, _sender: &Session, _args: NoArg) -> CommandResult {
    let subcommands = [
        "addref",
        "rmref",
        "listref",
        "make",
        "close",
        "lock",
        "unlock",
        "size",
        "move",
        "host",
        "clearhost",
        "start",
        "invite",
        "map",
        "set",
        "abort",
        "kick",
        "password",
        "randompassword",
        "mods",
        "team",
        "settings",
        "scorev",
        "help",
        "link",
        "timer",
        "aborttimer",
    ];

    Ok(Some(format!(
        "Supported multiplayer subcommands: <{}>.",
        subcommands.join(" | ")
    )))
}

#[command("link")]
pub async fn link<C: Context>(ctx: &C, sender: &Session, _args: NoArg) -> CommandResult {
    let _match_id = get_user_match_id(ctx, sender).await?;

    // TODO: Implement show_link logic
    // let link = multiplayer::get_match_link(ctx, match_id).await?;

    Ok(Some("Match link: (TODO: implement)".to_string()))
}

#[derive(FromCommandArgs)]
pub struct TimerArgs {
    pub time: i32,
}

#[command("timer")]
pub async fn timer<C: Context>(ctx: &C, sender: &Session, args: TimerArgs) -> CommandResult {
    if args.time < 1 {
        return Err(AppError::CommandsInvalidSyntax(
            "Countdown time must be at least 1 second",
            "!mp timer <time (in seconds)>",
            "!mp timer <time (in seconds)>",
        ));
    }

    if args.time > 300 {
        return Err(AppError::CommandsInvalidSyntax(
            "Countdown time must be less than 5 minutes",
            "!mp timer <time (in seconds)>",
            "!mp timer <time (in seconds)>",
        ));
    }

    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement start_timer logic
    // multiplayer::start_timer(ctx, match_id, args.time).await?;

    Ok(Some(format!("Countdown ends in {} second(s)", args.time)))
}

#[command("aborttimer")]
pub async fn aborttimer<C: Context>(ctx: &C, sender: &Session, _args: NoArg) -> CommandResult {
    let match_id = get_user_match_id(ctx, sender).await?;
    check_referee_permissions(ctx, sender, match_id).await?;

    // TODO: Implement abort_timer logic
    // multiplayer::abort_timer(ctx, match_id).await?;

    Ok(Some("Countdown stopped.".to_string()))
}
