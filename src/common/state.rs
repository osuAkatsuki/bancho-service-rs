use crate::common::context::Context;
use crate::common::redis_pool::RedisPool;
use sqlx::{MySql, Pool};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
}

impl AppState {
    pub fn new(db: Pool<MySql>, redis: RedisPool) -> Self {
        Self { db, redis }
    }

    pub fn from_ctx<C: Context>(ctx: &C) -> Self {
        Self {
            db: ctx.db_pool().clone(),
            redis: ctx.redis_pool().clone(),
        }
    }
}

impl Context for AppState {
    fn db_pool(&self) -> &Pool<MySql> {
        &self.db
    }

    fn redis_pool(&self) -> &RedisPool {
        &self.redis
    }
}
