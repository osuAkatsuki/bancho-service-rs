use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::error;

pub type ServiceResult<T> = Result<T, AppError>;
pub type ServiceResponse<T> = ServiceResult<Json<T>>;

#[track_caller]
pub fn unexpected<T, E: Into<anyhow::Error>>(e: E) -> ServiceResult<T> {
    let caller = std::panic::Location::caller();
    error!("An unexpected error has occurred at {caller}: {}", e.into());
    Err(AppError::Unexpected)
}

#[derive(Debug)]
pub enum AppError {
    Unexpected,
    Unauthorized,
    DecodingRequestFailed,
    InternalServerError(&'static str),
    UnsupportedClientVersion,
    ClientTooOld,
    InteractionBlocked,

    BeatmapsNotFound,

    ChannelsNotFound,
    ChannelsUnauthorized,
    ChannelsInvalidName,

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
    SessionsLimitReached,

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

    pub const fn code(&self) -> &'static str {
        match self {
            AppError::Unexpected => "unexpected",
            AppError::Unauthorized => "unauthorized",
            AppError::DecodingRequestFailed => "decoding_request_failed",
            AppError::InternalServerError(_) => "internal_server_error",
            AppError::UnsupportedClientVersion => "unsupported_client_version",
            AppError::ClientTooOld => "client_too_old",
            AppError::InteractionBlocked => "interaction_blocked",

            AppError::BeatmapsNotFound => "beatmaps.not_found",

            AppError::ChannelsNotFound => "channels.not_found",
            AppError::ChannelsUnauthorized => "channels.unauthorized",
            AppError::ChannelsInvalidName => "channels.invalid_name",

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
            AppError::SessionsLimitReached => "sessions.limit_reached",

            AppError::StreamsInvalidKey => "streams.invalid_key",
        }
    }

    pub const fn message(&self) -> &'static str {
        match self {
            AppError::Unexpected => "An unexpected error has occurred.",
            AppError::Unauthorized => "You are not authorized to perform this action.",
            AppError::DecodingRequestFailed => "Failed to decode request",
            AppError::InternalServerError(_) => "An internal server error has occurred.",
            AppError::UnsupportedClientVersion => "Client is unsupported",
            AppError::ClientTooOld => "Client is too old",
            AppError::InteractionBlocked => {
                "You do not have permission to interact with this user."
            }

            AppError::BeatmapsNotFound => "Beatmap could not be found.",

            AppError::ChannelsNotFound => "Channel not found",
            AppError::ChannelsUnauthorized => {
                "You do not have permission to send messages to this channel."
            }
            AppError::ChannelsInvalidName => "Invalid Channel Name (must start with `#`)",

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
            AppError::SessionsLimitReached => {
                "You have reached the max amount of logins. Please wait a few minutes or log out in other clients."
            }

            AppError::StreamsInvalidKey => "Invalid Streams Key",
        }
    }

    pub const fn http_status_code(&self) -> StatusCode {
        match self {
            AppError::DecodingRequestFailed
            | AppError::ChannelsInvalidName
            | AppError::CommandsInvalidSyntax(_, _, _)
            | AppError::MessagesInvalidLength
            | AppError::MultiplayerInvalidSlotID
            | AppError::StreamsInvalidKey => StatusCode::BAD_REQUEST,

            AppError::Unauthorized
            | AppError::ChannelsUnauthorized
            | AppError::CommandsUnauthorized
            | AppError::MultiplayerUnauthorized
            | AppError::MultiplayerInvalidPassword
            | AppError::SessionsInvalidCredentials => StatusCode::UNAUTHORIZED,

            AppError::UnsupportedClientVersion
            | AppError::ClientTooOld
            | AppError::InteractionBlocked
            | AppError::MultiplayerMatchFull
            | AppError::SessionsLoginForbidden
            | AppError::SessionsLimitReached => StatusCode::FORBIDDEN,

            AppError::BeatmapsNotFound
            | AppError::ChannelsNotFound
            | AppError::CommandsUnknownCommand
            | AppError::MultiplayerNotFound
            | AppError::MultiplayerSlotNotFound
            | AppError::MultiplayerUserNotInMatch
            | AppError::PresencesNotFound
            | AppError::RelationshipsNotFound
            | AppError::UsersNotFound
            | AppError::SessionsNotFound => StatusCode::NOT_FOUND,
            AppError::MessagesUserAutoSilenced => StatusCode::TOO_MANY_REQUESTS,

            AppError::Unexpected | AppError::InternalServerError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    pub const fn response_parts(&self) -> (StatusCode, Json<ErrorResponse>) {
        let status = self.http_status_code();
        let response = ErrorResponse {
            code: self.code(),
            message: self.message(),
        };
        (status, Json(response))
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.response_parts().into_response()
    }
}
