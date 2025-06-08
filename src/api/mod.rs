pub mod osu;
pub mod v1;

use crate::common::axum_ip::IpAddrInfo;
use crate::common::context::Context;
use crate::common::redis_pool::{PoolResult, RedisPool};
use crate::common::state::AppState;
use crate::models::bancho::BanchoResponse;
use crate::settings::AppSettings;
use async_trait::async_trait;
use axum::Router;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::routing::post;
use sqlx::{MySql, Pool};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

pub async fn serve(settings: &AppSettings, state: AppState) -> anyhow::Result<()> {
    let addr = SocketAddr::from((settings.app_host, settings.app_port));
    info!("Listening on {addr}");
    let listener = TcpListener::bind(addr).await?;
    let app = Router::new().merge(router()).with_state(state);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

#[derive(Clone)]
pub struct RequestContext {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
    pub request_ip: IpAddrInfo,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(osu::bancho::controller).get(osu::bancho::index))
        .nest("/api/v1", v1::router())
}

impl FromRequestParts<AppState> for RequestContext {
    type Rejection = BanchoResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ip_info = IpAddrInfo::from_request_parts(parts, state).await?;
        Ok(Self {
            db: state.db.clone(),
            redis: state.redis.clone(),
            request_ip: ip_info,
        })
    }
}

#[async_trait]
impl Context for RequestContext {
    fn db(&self) -> &Pool<MySql> {
        &self.db
    }

    async fn redis(&self) -> PoolResult {
        self.redis.get().await
    }
}
