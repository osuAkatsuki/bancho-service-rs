use crate::common::error::AppError;
use axum::extract::{FromRequest, Request};
use axum::response::{IntoResponse, Response};
use bancho_protocol::messages::Message;
use bancho_protocol::messages::server::Alert;
use std::str::FromStr;
use uuid::Uuid;

pub struct BanchoResponse {
    response_data: Vec<u8>,
    cho_token: String,
}

const DEFAULT_ERROR_TOKEN: &'static str = "no";
impl BanchoResponse {
    pub fn ok(session_id: Uuid, response_data: Vec<u8>) -> Self {
        Self {
            response_data,
            cho_token: session_id.to_string(),
        }
    }

    pub fn error_raw(session_id: Option<Uuid>, error: Vec<u8>) -> Self {
        let cho_token = session_id
            .map(|id| id.to_string())
            .unwrap_or(DEFAULT_ERROR_TOKEN.to_owned());
        Self {
            cho_token,
            response_data: error,
        }
    }

    pub fn error(session_id: Option<Uuid>, error: AppError) -> Self {
        let cho_token = session_id
            .map(|id| id.to_string())
            .unwrap_or(DEFAULT_ERROR_TOKEN.to_owned());
        Self {
            cho_token,
            response_data: Message::serialize(Alert {
                message: error.message(),
            }),
        }
    }

    pub fn extend(&mut self, data: Vec<u8>) {
        self.response_data.extend(data);
    }
}

pub enum BanchoRequest {
    Login(LoginArgs),
    HandleEvents(Uuid, axum::body::Bytes),
}

pub struct LoginArgs {
    pub identifier: String,
    pub secret: String,
    pub client_info: ClientInfo,
}

pub struct ClientInfo {
    pub osu_version: String,
    pub utc_offset: i8,
    pub display_city: bool,
    pub client_hashes: ClientHashes,
    pub pm_private: bool,
}

pub struct ClientHashes {
    pub osu_path_md5: String,
    pub adapters: NetworkAdapters,
    pub adapters_md5: String,
    pub uninstall_md5: String,
    pub disk_signature_md5: String,
}

pub struct NetworkAdapters {
    pub adapters: String,
}

impl IntoResponse for BanchoResponse {
    fn into_response(self) -> Response {
        IntoResponse::into_response(([("cho-token", self.cho_token)], self.response_data))
    }
}

const REQUEST_LIMIT: usize = 1024 * 1024 * 10;
impl<S: Send + Sync> FromRequest<S> for BanchoRequest {
    type Rejection = BanchoResponse;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        match req.headers().get("osu-token") {
            None => {
                let str = String::from_request(req, &())
                    .await
                    .map_err(|_| BanchoResponse::error(None, AppError::DecodingRequestFailed))?;
                let args = LoginArgs::from_str(&str).map_err(|e| BanchoResponse::error(None, e))?;
                Ok(BanchoRequest::Login(args))
            }
            Some(value) => {
                let token = value
                    .to_str()
                    .map_err(|_| BanchoResponse::error(None, AppError::DecodingRequestFailed))?;
                let token = Uuid::parse_str(token)
                    .map_err(|_| BanchoResponse::error(None, AppError::DecodingRequestFailed))?;
                let body = req.into_body();
                let request_data =
                    axum::body::to_bytes(body, REQUEST_LIMIT)
                        .await
                        .map_err(|_| {
                            BanchoResponse::error(Some(token), AppError::DecodingRequestFailed)
                        })?;
                Ok(BanchoRequest::HandleEvents(token, request_data))
            }
        }
    }
}

impl FromStr for LoginArgs {
    type Err = AppError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts = input.splitn(3, '\n');
        let identifier = parts
            .next()
            .ok_or(AppError::DecodingRequestFailed)?
            .to_string();
        let secret = parts
            .next()
            .ok_or(AppError::DecodingRequestFailed)?
            .to_string();
        let client_info_str = parts.next().ok_or(AppError::DecodingRequestFailed)?;
        let client_info = ClientInfo::from_str(client_info_str)?;

        Ok(Self {
            identifier,
            secret,
            client_info,
        })
    }
}

impl FromStr for ClientInfo {
    type Err = AppError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts = input.splitn(5, '|');
        let osu_version = parts
            .next()
            .ok_or(AppError::DecodingRequestFailed)?
            .to_string();
        let utc_offset_str = parts.next().ok_or(AppError::DecodingRequestFailed)?;
        let utc_offset =
            i8::from_str(utc_offset_str).map_err(|_| AppError::DecodingRequestFailed)?;
        let display_city = parts.next().ok_or(AppError::DecodingRequestFailed)? == "1";
        let client_hashes_str = parts.next().ok_or(AppError::DecodingRequestFailed)?;
        let client_hashes = ClientHashes::from_str(client_hashes_str)?;
        let pm_private = parts.next().ok_or(AppError::DecodingRequestFailed)? == "1";
        Ok(Self {
            osu_version,
            utc_offset,
            display_city,
            client_hashes,
            pm_private,
        })
    }
}

impl FromStr for ClientHashes {
    type Err = AppError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts = input.splitn(5, ':');
        let osu_path_md5 = parts
            .next()
            .ok_or(AppError::DecodingRequestFailed)?
            .to_string();
        let adapters_str = parts.next().ok_or(AppError::DecodingRequestFailed)?;
        let adapters = NetworkAdapters::from_str(adapters_str)?;
        let adapters_md5 = parts
            .next()
            .ok_or(AppError::DecodingRequestFailed)?
            .to_string();
        let uninstall_md5 = parts
            .next()
            .ok_or(AppError::DecodingRequestFailed)?
            .to_string();
        let disk_signature_md5 = parts
            .next()
            .ok_or(AppError::DecodingRequestFailed)?
            .to_string();
        Ok(Self {
            osu_path_md5,
            adapters,
            adapters_md5,
            uninstall_md5,
            disk_signature_md5,
        })
    }
}

impl FromStr for NetworkAdapters {
    type Err = AppError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(NetworkAdapters {
            adapters: input.to_string(),
        })
    }
}
