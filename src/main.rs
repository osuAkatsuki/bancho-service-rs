use bancho_service::settings::AppSettings;
use bancho_service::workers::{crons, daemons};
use bancho_service::{api, lifecycle};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = AppSettings::get();
    if settings.app_env == "dev" {
        lifecycle::initialize_logging(&settings);
    }

    match settings.app_component.as_str() {
        "api" => api::serve(settings).await,
        "cleanup-cron" => crons::cleanup_cron::serve(settings).await,
        "pubsub-daemon" => daemons::pubsub_consumer::serve(settings).await,
        _ => panic!("Unknown app component"),
    }
}
