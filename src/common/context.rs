use crate::common::redis_pool::{PoolResult, RedisPool};
use async_trait::async_trait;
use sqlx::{MySql, Pool};

pub trait Context: Sync + Send {
    fn db_pool(&self) -> &Pool<MySql>;
    fn redis_pool(&self) -> &RedisPool;
}

#[async_trait]
pub trait PoolContext: Sync + Send {
    fn db(&self) -> &Pool<MySql>;
    async fn redis(&self) -> PoolResult;
}

#[async_trait]
impl<T: Context> PoolContext for T {
    fn db(&self) -> &Pool<MySql> {
        self.db_pool()
    }

    async fn redis(&self) -> PoolResult {
        self.redis_pool().get().await
    }
}
