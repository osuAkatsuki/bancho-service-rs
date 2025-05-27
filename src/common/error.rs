use tracing::error;

pub type ServiceResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Unexpected,
    DecodingRequestFailed,
    InternalServerError(&'static str),
    ClientTooOld,

    ChannelsNotFound,
    ChannelsUnauthorized,

    PresencesNotFound,

    UsersNotFound,

    SessionsLoginForbidden,
    SessionsInvalidCredentials,
    SessionsNotFound,
    SessionsNeedsMigration,
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
            AppError::ClientTooOld => "client_too_old",

            AppError::ChannelsNotFound => "channels.not_found",
            AppError::ChannelsUnauthorized => "channels.unauthorized",

            AppError::PresencesNotFound => "presences.not_found",

            AppError::UsersNotFound => "users.not_found",

            AppError::SessionsLoginForbidden => "sessions.login_forbidden",
            AppError::SessionsInvalidCredentials => "sessions.invalid_credentials",
            AppError::SessionsNotFound => "sessions.not_found",
            AppError::SessionsNeedsMigration => "sessions.needs_migration",
        }
    }

    pub const fn message(&self) -> &str {
        match self {
            AppError::Unexpected => "An unexpected error has occurred.",
            AppError::DecodingRequestFailed => "Failed to decode request",
            AppError::InternalServerError(_) => "An internal server error has occurred.",
            AppError::ClientTooOld => "Client is too old",

            AppError::ChannelsNotFound => "Channel not found",
            AppError::ChannelsUnauthorized => "Unauthorized",

            AppError::PresencesNotFound => "Presence not found",

            AppError::UsersNotFound => "User not found",

            AppError::SessionsLoginForbidden => "Your account is not allowed to login.",
            AppError::SessionsInvalidCredentials => {
                "You have entered an invalid username or password."
            }
            AppError::SessionsNotFound => "Session not found",
            AppError::SessionsNeedsMigration => "Your session needs to be migrated.",
        }
    }
}

#[track_caller]
pub fn unexpected<T, E: Into<anyhow::Error>>(e: E) -> ServiceResult<T> {
    let caller = std::panic::Location::caller();
    error!("An unexpected error has occurred at {caller}: {}", e.into());
    Err(AppError::Unexpected)
}

macro_rules! unwrap_expect {
    (
        $e:expr
        $(, $($pat:pat => $result:expr),+ )?
    ) => {
        match $e {
            $( $($pat => $result,)+ )?
            Ok(v) => v,
            Err(e) => return $crate::common::error::unexpected(e),
        }
    };
}

pub(crate) use unwrap_expect;
