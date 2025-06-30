pub mod handlers;

use crate::common::init;
use crate::settings::AppSettings;
use crate::workers::daemons::pubsub_consumer::handlers::{
    ban, change_username, disconnect, notification, silence, unban, update_cached_stats, wipe,
};
use std::convert::Infallible;
use tracing::warn;

// TODO: change return type to anyhow::Result<!> when its stabilized
pub async fn serve(settings: &AppSettings) -> anyhow::Result<Infallible> {
    let redis_client = redis::Client::open(settings.redis_url.as_str())?;
    let mut redis_conn = redis_client.get_connection()?;
    let mut pubsub = redis_conn.as_pubsub();
    pubsub.subscribe("peppy:ban")?;
    pubsub.subscribe("peppy:unban")?;
    pubsub.subscribe("peppy:silence")?;
    pubsub.subscribe("peppy:disconnect")?;
    pubsub.subscribe("peppy:notification")?;
    pubsub.subscribe("peppy:change_username")?;
    pubsub.subscribe("peppy:update_cached_stats")?;
    pubsub.subscribe("peppy:wipe")?;

    let state = init::initialize_state(&settings).await?;
    loop {
        let msg = pubsub.get_message()?;
        let task_state = state.clone();
        tokio::spawn(async move {
            let msg = msg;
            let ctx = task_state;
            let channel_name = msg.get_channel_name();
            match channel_name {
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
            }
        });
    }
}
