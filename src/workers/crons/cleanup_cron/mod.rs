pub mod tasks;

use crate::settings::AppSettings;
use crate::{cron_tasks, lifecycle};
use tasks::cleanup_sessions::cleanup_sessions;
use tasks::cleanup_streams::cleanup_streams;

pub async fn serve(settings: &AppSettings) -> anyhow::Result<()> {
    let ctx = lifecycle::initialize_state(settings).await?;
    cron_tasks! {
        &ctx,
        cleanup_sessions,
        cleanup_streams,
    }
    Ok(())
}
