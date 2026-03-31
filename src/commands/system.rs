use crate::commands;
use crate::commands::{CommandResult, CommandRouterInstance};
use crate::common::context::Context;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::usecases::{bancho_settings, multiplayer, sessions};
use bancho_service_macros::command;
use tracing::error;

pub static COMMANDS: CommandRouterInstance = commands![maintenance];

#[command(
    "maintenance",
    required_privileges = Privileges::AdminCaker,
)]
pub async fn maintenance<C: Context>(ctx: &C, _sender: &Session) -> CommandResult {
    let is_active = bancho_settings::toggle_maintenance(ctx).await?;

    if is_active {
        let mut kicked_count = 0u64;
        let mut skipped_count = 0u64;

        let all_sessions: Vec<Session> = sessions::fetch_all(ctx).await?.collect();
        for session in &all_sessions {
            if session.privileges.is_staff() {
                continue;
            }

            let in_match = match multiplayer::fetch_session_match_id(ctx, session.session_id).await
            {
                Ok(Some(_)) => true,
                Ok(None) => false,
                Err(e) => {
                    error!(
                        user_id = session.user_id,
                        "Failed to check match status during maintenance kick: {e:?}"
                    );
                    true // err on the side of caution
                }
            };

            if in_match {
                skipped_count += 1;
                continue;
            }

            if let Err(e) = sessions::delete(ctx, session).await {
                error!(
                    user_id = session.user_id,
                    "Failed to kick user during maintenance: {e:?}"
                );
            } else {
                kicked_count += 1;
            }
        }

        Ok(Some(format!(
            "Turned on maintenance mode. Kicked {kicked_count} user(s), \
             skipped {skipped_count} in multiplayer."
        )))
    } else {
        Ok(Some("Turned off maintenance mode.".to_owned()))
    }
}
