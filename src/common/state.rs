use crate::api::RequestContext;
use crate::common::context::Context;
use crate::common::redis_pool::{PoolResult, RedisPool};
use sqlx::{MySql, Pool};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
}

impl Context for AppState {
    fn db(&self) -> &Pool<MySql> {
        &self.db
    }

    fn redis(&self) -> impl Future<Output = PoolResult> {
        self.redis.get()
    }
}
