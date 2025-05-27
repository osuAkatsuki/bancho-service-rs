use std::env;
use std::error::Error;
use std::str::FromStr;

pub trait FromEnv: Sized {
    fn from_env(env_var: &str) -> anyhow::Result<Self>;
}

impl<T: FromStr> FromEnv for T
where
    <T as FromStr>::Err: 'static + Error + Send + Sync,
{
    fn from_env(env_var: &str) -> anyhow::Result<Self> {
        let value = env::var(env_var)?;
        Ok(T::from_str(&value)?)
    }
}
