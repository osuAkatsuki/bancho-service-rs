use crate::commands::CommandResult;
use crate::common::context::Context;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::repositories::streams::StreamName;
use crate::usecases::{sessions, streams};
use bancho_protocol::messages::server::Alert;
use bancho_service_macros::{FromCommandArgs, command};

#[derive(Debug, FromCommandArgs)]
pub struct AlertUserArgs {
    pub username: String,
    pub message: String,
}

#[command(
    "alert",
    required_privileges = Privileges::AdminCaker,
)]
pub async fn alert_user<C: Context>(
    ctx: &C,
    _sender: &Session,
    args: AlertUserArgs,
) -> CommandResult {
    let session = sessions::fetch_primary_by_username(ctx, &args.username).await?;
    let alert = Alert {
        message: &args.message,
    };
    streams::broadcast_message(ctx, StreamName::User(session.session_id), alert, None, None)
        .await?;
    Ok("Alert sent successfully.".to_owned())
}
