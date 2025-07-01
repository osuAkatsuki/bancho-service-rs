use crate::common::error::ServiceResult;
use crate::common::state::AppState;
use crate::repositories::streams::StreamName;
use crate::usecases::{sessions, streams, users};
use bancho_protocol::messages::server::{SilenceEnd, UserSilenced};
use chrono::Utc;
use redis::Msg;
use tracing::info;

pub async fn handle(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let user_id: i64 = msg.get_payload()?;
    info!(user_id, "Handling silence event for user");

    let user = users::fetch_one(&ctx, user_id).await?;
    let seconds_left = match user.silence_end {
        None => 0,
        Some(silence_end) => (Utc::now() - silence_end).num_seconds(),
    };
    let sessions = sessions::fetch_by_user_id(&ctx, user_id).await?;
    for mut session in sessions {
        session.silence_end = user.silence_end;
        let session = sessions::update(&ctx, session).await?;

        let silence_end = SilenceEnd {
            seconds_left: seconds_left as _,
        };
        streams::broadcast_message(
            &ctx,
            StreamName::User(session.session_id),
            silence_end,
            None,
            None,
        )
        .await?;
    }

    streams::broadcast_message(
        &ctx,
        StreamName::Main,
        UserSilenced {
            user_id: user.user_id as _,
        },
        None,
        None,
    )
    .await?;

    info!(user_id, "Successfully handled silence event for user");
    Ok(())
}
