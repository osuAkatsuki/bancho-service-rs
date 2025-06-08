use axum::Router;
use bancho_service::api;
use bancho_service::common::init;
use bancho_service::settings::AppSettings;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = AppSettings::get();
    init::initialize_logging(&settings);

    info!("Hello, world!");
    let state = init::initialize_state(&settings).await?;

    let addr = SocketAddr::from((settings.app_host, settings.app_port));
    let listener = TcpListener::bind(addr).await?;
    let app = Router::new().merge(api::router()).with_state(state);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
