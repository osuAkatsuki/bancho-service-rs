use crate::common::context::Context;
use crate::common::redis_json::Json;
use crate::entities::tillerino::LastNowPlayingState;
use redis::{AsyncCommands, HashFieldExpirationOptions, SetExpiry};
use uuid::Uuid;

const KEY: &str = "akatsuki:bancho:tillerino";

pub async fn save_np<C: Context>(
    ctx: &C,
    session_id: Uuid,
    last_np: LastNowPlayingState,
) -> anyhow::Result<LastNowPlayingState> {
    let mut redis = ctx.redis().await?;
    let expiration = HashFieldExpirationOptions::default().set_expiration(SetExpiry::EX(600));
    let _: () = redis
        .hset_ex(KEY, &expiration, &[(session_id, Json(&last_np))])
        .await?;
    Ok(last_np)
}

pub async fn fetch_last_np<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> anyhow::Result<LastNowPlayingState> {
    let mut redis = ctx.redis().await?;
    let last_np: Json<LastNowPlayingState> = redis.hget(KEY, session_id).await?;
    Ok(last_np.into_inner())
}
