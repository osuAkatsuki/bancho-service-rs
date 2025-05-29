use crate::common::context::Context;
use crate::common::redis_pool::{PoolResult, RedisPool};
use async_trait::async_trait;
use sqlx::{MySql, Pool};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
}

#[async_trait]
impl Context for AppState {
    fn db(&self) -> &Pool<MySql> {
        &self.db
    }

    async fn redis(&self) -> PoolResult {
        self.redis.get().await
    }
}
