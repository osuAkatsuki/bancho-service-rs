use crate::common::env::FromEnv;
use std::env;
use std::net::IpAddr;
use std::ops::Deref;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::Level;

pub struct AppSettings {
    pub app_component: String,
    pub level: Level,
    pub app_host: IpAddr,
    pub app_port: u16,

    pub database_url: String,
    pub db_max_connections: usize,
    pub db_wait_timeout: Duration,

    pub redis_url: String,
    pub redis_max_connections: usize,
    pub redis_connection_timeout: Duration,
    pub redis_response_timeout: Duration,
    pub redis_wait_timeout: Duration,

    pub discord_webhook_url: Option<String>,
}

impl AppSettings {
    pub fn load_from_env() -> anyhow::Result<Self> {
        let _ = dotenv::dotenv();

        let app_component = env::var("APP_COMPONENT")?;
        let level = Level::from_env("LOG_LEVEL")?;
        let app_host = IpAddr::from_env("APP_HOST")?;
        let app_port = u16::from_env("APP_PORT")?;

        let database_url = env::var("DATABASE_URL")?;
        let db_max_connections = usize::from_env("DB_MAX_CONNECTIONS")?;
        let db_wait_timeout_secs = u64::from_env("DB_WAIT_TIMEOUT_SECS")?;
        let db_wait_timeout = Duration::from_secs(db_wait_timeout_secs);

        let redis_url = env::var("REDIS_URL")?;
        let redis_max_connections = usize::from_env("REDIS_MAX_CONNECTIONS")?;
        let redis_connection_timeout_secs = u64::from_env("REDIS_CONNECTION_TIMEOUT_SECS")?;
        let redis_connection_timeout = Duration::from_secs(redis_connection_timeout_secs);
        let redis_response_timeout_secs = u64::from_env("REDIS_RESPONSE_TIMEOUT_SECS")?;
        let redis_response_timeout = Duration::from_secs(redis_response_timeout_secs);
        let redis_wait_timeout_secs = u64::from_env("REDIS_WAIT_TIMEOUT_SECS")?;
        let redis_wait_timeout = Duration::from_secs(redis_wait_timeout_secs);

        let discord_webhook_url = env::var("DISCORD_WEBHOOK_URL").ok();

        Ok(AppSettings {
            app_component,
            level,
            app_port,
            app_host,

            database_url,
            db_max_connections,
            db_wait_timeout,

            redis_url,
            redis_max_connections,
            redis_connection_timeout,
            redis_response_timeout,
            redis_wait_timeout,

            discord_webhook_url,
        })
    }

    pub fn get() -> &'static AppSettings {
        settings()
    }
}

pub fn settings() -> &'static AppSettings {
    static SETTINGS: LazyLock<AppSettings> =
        LazyLock::new(|| AppSettings::load_from_env().expect("Failed to load settings"));
    SETTINGS.deref()
}
