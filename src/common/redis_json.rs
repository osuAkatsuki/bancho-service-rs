use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs, Value};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[repr(transparent)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Serialize> ToRedisArgs for Json<T> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        let json_encoded = serde_json::to_string(&self.0).expect("Failed to serialize JSON");
        json_encoded.write_redis_args(out);
    }
}

impl<T: for<'a> Deserialize<'a>> FromRedisValue for Json<T> {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let json_encoded = String::from_redis_value(v)?;
        let json_decoded: T =
            serde_json::from_str(&json_encoded).map_err(redis::RedisError::from)?;
        Ok(Json(json_decoded))
    }
}

impl<T: Default> Default for Json<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<T: Debug> Debug for Json<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}
