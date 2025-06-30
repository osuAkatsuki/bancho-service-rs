pub mod tasks;

use crate::common::init;
use crate::cron_tasks;
use crate::settings::AppSettings;
use tasks::cleanup_sessions::cleanup_sessions;
use tasks::cleanup_streams::cleanup_streams;

pub async fn serve(settings: &AppSettings) -> anyhow::Result<()> {
    let ctx = init::initialize_state(settings).await?;
    cron_tasks! {
        &ctx,
        cleanup_sessions,
        cleanup_streams,
    }
    Ok(())
}
