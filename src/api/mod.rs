use crate::common::axum_ip::IpAddrInfo;
use crate::common::context::Context;
use crate::common::redis_pool::{PoolResult, RedisPool};
use crate::common::state::AppState;
use crate::models::bancho::BanchoResponse;
use async_trait::async_trait;
use axum::Router;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::routing::post;
use sqlx::{MySql, Pool};

pub mod osu;
pub mod v1;

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
