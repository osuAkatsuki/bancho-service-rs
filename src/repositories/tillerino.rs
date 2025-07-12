use crate::common::context::Context;
use crate::common::redis_json::Json;
use crate::entities::tillerino::NowPlayingState;
use redis::{AsyncCommands, SetExpiry, SetOptions};
use uuid::Uuid;

const EXPIRY: SetExpiry = SetExpiry::EX(600);

fn make_key(session_id: Uuid) -> String {
    format!("akatsuki:bancho:tillerino:{session_id}")
}

pub async fn save_np<C: Context>(
    ctx: &C,
    session_id: Uuid,
    last_np: NowPlayingState,
) -> anyhow::Result<NowPlayingState> {
    let mut redis = ctx.redis().await?;
    let key = make_key(session_id);
    let opts = SetOptions::default().with_expiration(EXPIRY);
    let _: () = redis.set_options(key, Json(&last_np), opts).await?;
    Ok(last_np)
}

pub async fn fetch_last_np<C: Context>(
    ctx: &C,
    session_id: Uuid,
) -> anyhow::Result<Option<NowPlayingState>> {
    let mut redis = ctx.redis().await?;
    let key = make_key(session_id);
    let last_np: Option<Json<NowPlayingState>> = redis.get(key).await?;
    Ok(last_np.map(Json::into_inner))
}
