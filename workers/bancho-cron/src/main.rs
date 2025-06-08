mod tasks;

use crate::tasks::cleanup_sessions::cleanup_sessions;
use crate::tasks::cleanup_streams::cleanup_streams;
use bancho_service::common::init;
use bancho_service::settings::AppSettings;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = AppSettings::get();
    init::initialize_logging(&settings);
    let ctx = init::initialize_state(&settings).await?;
    cron_tasks! {
        &ctx,
        cleanup_sessions,
        cleanup_streams,
    }
    Ok(())
}
