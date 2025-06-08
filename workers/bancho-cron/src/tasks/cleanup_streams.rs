use bancho_service::common::context::Context;
use bancho_service::common::error::ServiceResult;
use bancho_service::repositories::streams::StreamName;
use bancho_service::usecases::streams;
use chrono::{TimeDelta, Utc};
use std::ops::Add;
use tracing::{error, info};

const MESSAGE_TTL: usize = 5 * 60;
const CLEAR_STREAM_INTERVAL: i64 = 10 * 60;

pub async fn cleanup_streams<C: Context>(ctx: &C) -> ServiceResult<()> {
    let streams = streams::fetch_all(ctx).await?;
    for key in streams {
        match cleanup_stream(ctx, key).await {
            Ok((key, count)) => match count {
                usize::MAX => info!("Cleared Stream {key}"),
                count => info!("Trimmed {count} messages from {key}"),
            },
            Err(e) => error!("Cleanup stream error: {e:?}"),
        }
    }

    Ok(())
}

async fn cleanup_stream<C: Context>(ctx: &C, key: String) -> ServiceResult<(String, usize)> {
    let stream = StreamName::from_key(&key)?;
    let timestamp = streams::get_latest_message_timestamp(ctx, stream).await?;
    let now = Utc::now();

    // If there was no message broadcasted in the interval, fully delete the stream
    if timestamp.add(TimeDelta::seconds(CLEAR_STREAM_INTERVAL)) < now {
        streams::clear_stream(ctx, stream).await?;
        Ok((key, usize::MAX))
    } else {
        let count = streams::trim_stream(ctx, stream, MESSAGE_TTL).await?;
        Ok((key, count))
    }
}
