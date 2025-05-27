use crate::common::redis_pool::PoolResult;
use sqlx::{MySql, Pool};

pub trait Context {
    fn db(&self) -> &Pool<MySql>;
    fn redis(&self) -> impl Future<Output = PoolResult>;
}
