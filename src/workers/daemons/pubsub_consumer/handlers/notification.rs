use crate::common::error::ServiceResult;
use crate::common::redis_json::Json;
use crate::common::state::AppState;
use crate::repositories::streams::StreamName;
use crate::usecases::{sessions, streams};
use bancho_protocol::messages::server::Alert;
use redis::Msg;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
struct NotificationArgs {
    #[serde(rename = "userID")]
    pub user_id: i64,
    pub message: String,
}

pub async fn handle(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let args: Json<NotificationArgs> = msg.get_payload()?;
    let args = args.into_inner();

    info!(
        user_id = args.user_id,
        message = args.message,
        "Handling notification event for user"
    );

    let session = sessions::fetch_primary_by_user_id(&ctx, args.user_id).await?;
    let notification = Alert {
        message: &args.message,
    };
    streams::broadcast_message(
        &ctx,
        StreamName::User(session.session_id),
        notification,
        None,
        None,
    )
    .await?;

    info!(
        user_id = args.user_id,
        message = args.message,
        "Successfully handled notification event for user"
    );
    Ok(())
}
