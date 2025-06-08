pub mod tasks;

use crate::common::state::AppState;
use crate::cron_tasks;
use tasks::cleanup_sessions::cleanup_sessions;
use tasks::cleanup_streams::cleanup_streams;

pub async fn serve(ctx: AppState) -> anyhow::Result<()> {
    cron_tasks! {
        &ctx,
        cleanup_sessions,
        cleanup_streams,
    }
    Ok(())
}
