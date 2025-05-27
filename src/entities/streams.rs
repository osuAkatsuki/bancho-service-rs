use redis::{FromRedisValue, RedisError, RedisResult, Value};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

pub struct StreamMessage<'a> {
    data: &'a [u8],
    info: String,
}

impl<'a> StreamMessage<'a> {
    pub fn new(data: &'a [u8], info: MessageInfo) -> Self {
        let info = serde_json::to_string(&info).unwrap();
        Self { data, info }
    }

    pub fn items(&'a self) -> [(&'static str, &'a [u8]); 2] {
        [("data", &self.data), ("info", self.info.as_bytes())]
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageInfo {
    pub excluded_session_ids: Option<Vec<Uuid>>,
    pub read_privileges: Option<i32>,
}

#[derive(Debug)]
pub struct StreamReadReply {
    pub streams: Vec<StreamReply>,
}

#[derive(Debug)]
pub struct StreamReply {
    pub stream_name: String,
    pub messages: Vec<StreamReadMessage>,
}

#[derive(Debug)]
pub struct StreamReadMessage {
    pub message_id: String,
    pub data: Vec<u8>,
    pub info: MessageInfo,
}

macro_rules! array {
    ($e:expr) => {
        match $e {
            Value::Array(a) => a,
            v => return Err(invalid_stream_response(v)),
        }
    };
}

fn process_stream_message(message: &Value) -> RedisResult<StreamReadMessage> {
    let message = array!(message);
    let message_id = String::from_redis_value(&message[0])?;

    let mut data = None;
    let mut info = None;
    for chunk in array!(&message[1]).chunks_exact(2) {
        let key = String::from_redis_value(&chunk[0])?;
        let value = <Vec<u8>>::from_redis_value(&chunk[1])?;
        if key == "data" {
            data = Some(value);
        } else if key == "info" {
            info = Some(serde_json::from_slice(&value).expect("failed to deserialize info"));
        } else {
            warn!("Unknown redis stream message key: {key}");
        }
    }
    let data = data.unwrap();
    let info = info.unwrap();
    Ok(StreamReadMessage {
        message_id,
        data,
        info,
    })
}

fn process_stream_reply(reply: &Vec<Value>) -> RedisResult<StreamReply> {
    let stream_name = String::from_redis_value(&reply[0])?;
    let messages = array!(&reply[1]);
    let messages = messages
        .iter()
        .map(process_stream_message)
        .collect::<RedisResult<Vec<_>>>()?;
    Ok(StreamReply {
        stream_name,
        messages,
    })
}

impl FromRedisValue for StreamReadReply {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let stream_replies = array!(v);
        let mut streams = vec![];
        for reply in stream_replies {
            let reply = array!(reply);
            let stream = process_stream_reply(reply)?;
            streams.push(stream);
        }

        Ok(StreamReadReply { streams })
    }
}

fn invalid_stream_response(value: &Value) -> RedisError {
    redis::RedisError::from((
        redis::ErrorKind::TypeError,
        "Response was of incompatible type",
        format!(
            "Response type not string compatible. (response was {:?})",
            value
        ),
    ))
}
