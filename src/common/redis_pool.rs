use deadpool::managed::{Manager, Metrics, Object, Pool, PoolError, RecycleResult};
use redis::{AsyncCommands, AsyncConnectionConfig, RedisError, RedisResult};

pub struct RedisPoolManager {
    client: redis::Client,
    config: AsyncConnectionConfig,
}

impl RedisPoolManager {
    pub fn new(client: redis::Client, config: AsyncConnectionConfig) -> Self {
        Self { client, config }
    }
}

impl Manager for RedisPoolManager {
    type Type = redis::aio::MultiplexedConnection;
    type Error = RedisError;

    async fn create(&self) -> RedisResult<Self::Type> {
        self.client
            .get_multiplexed_async_connection_with_config(&self.config)
            .await
    }

    // TODO: maybe trace metrics
    async fn recycle(
        &self,
        connection: &mut Self::Type,
        _metrics: &Metrics,
    ) -> RecycleResult<Self::Error> {
        let _: () = connection.ping().await?;
        Ok(())
    }
}

pub type RedisPool = Pool<RedisPoolManager>;
pub type Connection = Object<RedisPoolManager>;
pub type Error = PoolError<RedisError>;
pub type PoolResult = Result<Connection, Error>;
