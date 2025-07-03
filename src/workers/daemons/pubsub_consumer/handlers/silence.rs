use crate::common::error::ServiceResult;
use crate::common::state::AppState;
use crate::repositories::streams::StreamName;
use crate::usecases::{sessions, streams, users};
use bancho_protocol::messages::Message;
use bancho_protocol::messages::server::{SilenceEnd, UserSilenced};
use chrono::Utc;
use redis::Msg;
use tracing::info;

pub async fn handle(ctx: AppState, msg: Msg) -> ServiceResult<()> {
    let user_id: i64 = msg.get_payload()?;
    info!(user_id, "Handling silence event for user");

    let user = users::fetch_one(&ctx, user_id).await?;
    let is_silenced = user.is_silenced();
    let seconds_left = match is_silenced {
        false => 0,
        true => (user.silence_end.unwrap() - Utc::now()).num_seconds(),
    };
    let silence_end = Message::serialize(SilenceEnd {
        seconds_left: seconds_left as _,
    });

    let sessions = sessions::fetch_by_user_id(&ctx, user_id).await?;
    for mut session in sessions {
        session.silence_end = user.silence_end;
        let session = sessions::update(&ctx, session).await?;
        streams::broadcast_data(
            &ctx,
            StreamName::User(session.session_id),
            &silence_end,
            None,
            None,
        )
        .await?;
    }

    if is_silenced {
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
    }

    info!(user_id, "Successfully handled silence event for user");
    Ok(())
}
