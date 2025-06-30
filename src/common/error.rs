use tracing::error;

pub type ServiceResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Unexpected,
    DecodingRequestFailed,
    InternalServerError(&'static str),
    UnsupportedClientVersion,
    ClientTooOld,
    InteractionBlocked,

    ChannelsNotFound,
    ChannelsUnauthorized,

    /// 0: Syntax, 1: Type Signature, 2: Typed Syntax
    CommandsInvalidSyntax(&'static str, &'static str, &'static str),
    CommandsUnknownCommand,
    CommandsUnauthorized,

    MessagesInvalidLength,
    MessagesUserAutoSilenced,

    MultiplayerNotFound,
    MultiplayerUnauthorized,
    MultiplayerInvalidPassword,
    MultiplayerMatchFull,
    MultiplayerInvalidSlotID,
    MultiplayerSlotNotFound,
    MultiplayerUserNotInMatch,

    PresencesNotFound,

    RelationshipsNotFound,

    UsersNotFound,

    SessionsLoginForbidden,
    SessionsInvalidCredentials,
    SessionsNotFound,
    SessionsNeedsMigration,

    StreamsInvalidKey,
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    #[track_caller]
    fn from(e: E) -> Self {
        unexpected::<(), E>(e).unwrap_err()
    }
}

impl AppError {
    pub const fn as_str(&self) -> &str {
        self.code()
    }

    pub const fn code(&self) -> &str {
        match self {
            AppError::Unexpected => "unexpected",
            AppError::DecodingRequestFailed => "decoding_request_failed",
            AppError::InternalServerError(_) => "internal_server_error",
            AppError::UnsupportedClientVersion => "unsupported_client_version",
            AppError::ClientTooOld => "client_too_old",
            AppError::InteractionBlocked => "interaction_blocked",

            AppError::ChannelsNotFound => "channels.not_found",
            AppError::ChannelsUnauthorized => "channels.unauthorized",

            AppError::CommandsInvalidSyntax(_, _, _) => "commands.invalid_syntax",
            AppError::CommandsUnknownCommand => "commands.unknown_command",
            AppError::CommandsUnauthorized => "commands.unauthorized",

            AppError::MessagesInvalidLength => "messages.invalid_length",
            AppError::MessagesUserAutoSilenced => "messages.user_auto_silenced",

            AppError::MultiplayerNotFound => "multiplayer.not_found",
            AppError::MultiplayerUnauthorized => "multiplayer.unauthorized",
            AppError::MultiplayerInvalidPassword => "multiplayer.invalid_password",
            AppError::MultiplayerMatchFull => "multiplayer.match_full",
            AppError::MultiplayerInvalidSlotID => "multiplayer.invalid_slot_id",
            AppError::MultiplayerSlotNotFound => "multiplayer.slot_not_found",
            AppError::MultiplayerUserNotInMatch => "multiplayer.user_not_in_match",

            AppError::PresencesNotFound => "presences.not_found",

            AppError::RelationshipsNotFound => "relationships.not_found",

            AppError::UsersNotFound => "users.not_found",

            AppError::SessionsLoginForbidden => "sessions.login_forbidden",
            AppError::SessionsInvalidCredentials => "sessions.invalid_credentials",
            AppError::SessionsNotFound => "sessions.not_found",
            AppError::SessionsNeedsMigration => "sessions.needs_migration",

            AppError::StreamsInvalidKey => "streams.invalid_key",
        }
    }

    pub const fn message(&self) -> &str {
        match self {
            AppError::Unexpected => "An unexpected error has occurred.",
            AppError::DecodingRequestFailed => "Failed to decode request",
            AppError::InternalServerError(_) => "An internal server error has occurred.",
            AppError::UnsupportedClientVersion => "Client is unsupported",
            AppError::ClientTooOld => "Client is too old",
            AppError::InteractionBlocked => {
                "You do not have permission to interact with this user."
            }

            AppError::ChannelsNotFound => "Channel not found",
            AppError::ChannelsUnauthorized => {
                "You do not have permission to send messages to this channel."
            }

            AppError::CommandsInvalidSyntax(_, _, _) => "Invalid Command Syntax",
            AppError::CommandsUnknownCommand => "Unknown Command",
            AppError::CommandsUnauthorized => {
                "You do not have sufficient privileges to use this command."
            }

            AppError::MessagesInvalidLength => {
                "Your message was too short/long. It has not been sent."
            }
            AppError::MessagesUserAutoSilenced => {
                "You have sent too many messages in a short period of time."
            }

            AppError::MultiplayerNotFound => "The multiplayer match could not be found.",
            AppError::MultiplayerUnauthorized => {
                "You do not have sufficient privileges to perform this action."
            }
            AppError::MultiplayerInvalidPassword => {
                "You have entered an invalid password for this match."
            }
            AppError::MultiplayerMatchFull => "The match has no free space left.",
            AppError::MultiplayerInvalidSlotID => "The slot id is invalid.",
            AppError::MultiplayerSlotNotFound => "The slot could not be found.",
            AppError::MultiplayerUserNotInMatch => "The user is not in this match.",

            AppError::PresencesNotFound => "Presence not found",

            AppError::RelationshipsNotFound => "Relationship not found",

            AppError::UsersNotFound => "This user does not exist.",

            AppError::SessionsLoginForbidden => "Your account is not allowed to login.",
            AppError::SessionsInvalidCredentials => {
                "You have entered an invalid username or password."
            }
            AppError::SessionsNotFound => "This user is currently not online",
            AppError::SessionsNeedsMigration => "Your session needs to be migrated.",

            AppError::StreamsInvalidKey => "Invalid Streams Key",
        }
    }
}

#[track_caller]
pub fn unexpected<T, E: Into<anyhow::Error>>(e: E) -> ServiceResult<T> {
    let caller = std::panic::Location::caller();
    error!("An unexpected error has occurred at {caller}: {}", e.into());
    Err(AppError::Unexpected)
}
