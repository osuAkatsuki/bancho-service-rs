use crate::common::error::ServiceResult;
use crate::common::redis_json::Json;
use crate::common::state::AppState;
use crate::usecases::sessions;
use redis::Msg;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
struct DisconnectArgs {
    #[serde(rename = "userID")]
    pub user_id: i64,
    pub reason: String,
}

pub async fn handle(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let args: Json<DisconnectArgs> = msg.get_payload()?;
    let args = args.into_inner();

    info!(
        user_id = args.user_id,
        reason = args.reason,
        "Handling disconnect event for user"
    );

    let sessions = sessions::fetch_by_user_id(&ctx, args.user_id).await?;
    for session in sessions {
        sessions::delete(&ctx, &session).await?;
    }

    info!(
        user_id = args.user_id,
        reason = args.reason,
        "Successfully handled disconnect event for user"
    );
    Ok(())
}
