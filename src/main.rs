use crate::common::redis_pool::{RedisPool, RedisPoolManager};
use crate::common::state::AppState;
use crate::settings::AppSettings;
use axum::Router;
use deadpool::Runtime;
use redis::AsyncConnectionConfig;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

mod adapters;
mod api;
mod common;
mod entities;
mod events;
mod models;
mod repositories;
mod settings;
mod usecases;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = AppSettings::get();
    initialize_logging(&settings);

    info!("Hello, world!");
    let db = initialize_db(&settings).await?;
    let redis = initialize_redis(&settings)?;
    let state = AppState { db, redis };

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

fn initialize_logging(settings: &AppSettings) {
    tracing_subscriber::fmt()
        .with_max_level(settings.level)
        // .json()
        .with_timer(tracing_subscriber::fmt::time())
        .with_level(true)
        .compact()
        .init();
}

fn initialize_db(settings: &AppSettings) -> impl Future<Output = sqlx::Result<Pool<MySql>>> {
    MySqlPoolOptions::new()
        .acquire_timeout(settings.db_wait_timeout)
        .max_connections(settings.db_max_connections as _)
        .connect(&settings.database_url)
}

fn initialize_redis(settings: &AppSettings) -> anyhow::Result<RedisPool> {
    let redis_client = redis::Client::open(settings.redis_url.as_str())?;
    let redis_cfg = AsyncConnectionConfig::new()
        .set_connection_timeout(settings.redis_connection_timeout)
        .set_response_timeout(settings.redis_response_timeout);

    let redis_manager = RedisPoolManager::new(redis_client, redis_cfg);
    let redis = RedisPool::builder(redis_manager)
        .max_size(settings.redis_max_connections)
        .wait_timeout(Some(settings.redis_wait_timeout))
        .runtime(Runtime::Tokio1)
        .build()?;
    Ok(redis)
}
