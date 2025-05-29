use crate::common::redis_pool::PoolResult;
use async_trait::async_trait;
use sqlx::{MySql, Pool};

#[async_trait]
pub trait Context: Sync + Send {
    fn db(&self) -> &Pool<MySql>;
    async fn redis(&self) -> PoolResult;
}
