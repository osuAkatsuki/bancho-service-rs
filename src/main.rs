use bancho_service::api;
use bancho_service::common::init;
use bancho_service::settings::AppSettings;
use bancho_service::workers::crons;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = AppSettings::get();
    init::initialize_logging(&settings);
    match settings.app_component.as_str() {
        "api" => api::serve(settings).await,
        "cleanup-cron" => crons::cleanup_cron::serve(settings).await,
        _ => panic!("Unknown app component"),
    }
}
