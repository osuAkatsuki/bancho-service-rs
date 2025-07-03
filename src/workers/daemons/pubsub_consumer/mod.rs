pub mod handlers;

use crate::lifecycle;
use crate::settings::AppSettings;
use crate::workers::daemons::pubsub_consumer::handlers::{
    ban, change_username, disconnect, notification, silence, unban, update_cached_stats, wipe,
};
use tracing::{error, info, warn};

pub const PUBSUB_CHANNELS: [&str; 8] = [
    "peppy:ban",
    "peppy:unban",
    "peppy:silence",
    "peppy:disconnect",
    "peppy:notification",
    "peppy:change_username",
    "peppy:update_cached_stats",
    "peppy:wipe",
];

// TODO: change return type to anyhow::Result<!> when its stabilized
pub async fn serve(settings: &AppSettings) -> anyhow::Result<()> {
    let redis_client = redis::Client::open(settings.redis_url.as_str())?;
    let mut redis_conn = redis_client.get_connection()?;
    let mut pubsub = redis_conn.as_pubsub();
    for channel in PUBSUB_CHANNELS {
        info!(channel, "Subscribing to pubsub channel");
        pubsub.subscribe(channel)?;
    }

    let state = lifecycle::initialize_state(&settings).await?;
    loop {
        let msg = pubsub.get_message()?;
        let task_state = state.clone();
        tokio::spawn(async move {
            let msg = msg;
            let ctx = task_state;
            let channel_name = msg.get_channel_name().to_string();
            let handler_result = match channel_name.as_str() {
                "peppy:ban" => ban::handle(ctx, msg).await,
                "peppy:unban" => unban::handle(ctx, msg).await,
                "peppy:silence" => silence::handle(ctx, msg).await,
                "peppy:disconnect" => disconnect::handle(ctx, msg).await,
                "peppy:notification" => notification::handle(ctx, msg).await,
                "peppy:change_username" => change_username::handle(ctx, msg).await,
                "peppy:update_cached_stats" => update_cached_stats::handle(ctx, msg).await,
                "peppy:wipe" => wipe::handle(ctx, msg).await,
                _ => {
                    warn!("Unknown pubsub channel message: {}", channel_name);
                    Ok(())
                }
            };
            if let Err(e) = handler_result {
                error!(channel_name, "Error handling pubsub event: {e:?}");
            }
        });
    }
}
