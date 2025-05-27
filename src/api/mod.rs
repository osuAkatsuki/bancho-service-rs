use crate::common::axum_ip::IpAddrInfo;
use crate::common::redis_pool::RedisPool;
use crate::common::state::AppState;
use crate::models::bancho::BanchoResponse;
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
