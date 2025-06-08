use crate::common::error::AppError;
use axum::extract::{FromRequest, Request};
use axum::response::{IntoResponse, Response};
use bancho_protocol::messages::Message;
use bancho_protocol::messages::server::Alert;
use chrono::{Months, NaiveDate};
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

#[derive(Debug)]
pub struct LoginArgs {
    pub identifier: String,
    pub secret: String,
    pub client_info: ClientInfo,
}

#[derive(Debug)]
pub struct ClientInfo {
    pub osu_version: OsuVersion,
    pub utc_offset: i8,
    pub display_city: bool,
    pub client_hashes: ClientHashes,
    pub pm_private: bool,
}

#[derive(Debug)]
pub enum ReleaseStream {
    Stable,
    Beta,
    CuttingEdge,
}

#[derive(Debug)]
pub struct OsuVersion {
    pub release_stream: ReleaseStream,
    pub version_date: NaiveDate,
    pub version_minor: Option<i32>,
}

impl OsuVersion {
    pub fn is_outdated(&self) -> bool {
        let today = chrono::Utc::now().date_naive();
        let version_expiration_months = match self.release_stream {
            ReleaseStream::Stable => 12,
            ReleaseStream::Beta => 12,
            ReleaseStream::CuttingEdge => 6,
        };
        let version_expiration_date = self.version_date + Months::new(version_expiration_months);
        // Version is outdated
        today > version_expiration_date
    }
}

impl FromStr for OsuVersion {
    type Err = AppError;

    fn from_str(version_string: &str) -> Result<Self, Self::Err> {
        let version_string = version_string
            .strip_prefix("b")
            .ok_or(AppError::UnsupportedClientVersion)?;
        let (beta, version_string) = match version_string.strip_suffix("beta") {
            None => (false, version_string),
            Some(version_string) => (true, version_string),
        };
        let (cutting_edge, version_string) = match version_string.strip_suffix("cuttingedge") {
            None => (false, version_string),
            Some(version_string) => (true, version_string),
        };
        let release_stream = if beta {
            ReleaseStream::Beta
        } else if cutting_edge {
            ReleaseStream::CuttingEdge
        } else {
            ReleaseStream::Stable
        };

        let mut version_split = version_string.split('.');
        let version_date = version_split
            .next()
            .ok_or(AppError::UnsupportedClientVersion)?;
        let version_minor = version_split.next();

        let version_date = NaiveDate::parse_from_str(version_date, "%Y%m%d")
            .map_err(|_| AppError::UnsupportedClientVersion)?;
        let version_minor = match version_minor {
            None => None,
            Some(version_minor) => {
                let minor =
                    i32::from_str(version_minor).map_err(|_| AppError::UnsupportedClientVersion)?;
                Some(minor)
            }
        };
        Ok(Self {
            release_stream,
            version_date,
            version_minor,
        })
    }
}

#[derive(Debug)]
pub struct ClientHashes {
    pub osu_path_md5: String,
    pub adapters: NetworkAdapters,
    pub adapters_md5: String,
    pub uninstall_md5: String,
    pub disk_signature_md5: String,
}

#[derive(Debug)]
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
        let osu_version_str = parts.next().ok_or(AppError::DecodingRequestFailed)?;
        let osu_version = OsuVersion::from_str(osu_version_str)?;
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
        let mut parts = input.splitn(6, ':');
        let osu_path_md5 = parts
            .next()
            .filter(|path_md5| path_md5.len() == 32)
            .ok_or(AppError::DecodingRequestFailed)?
            .to_string();
        let adapters_str = parts.next().ok_or(AppError::DecodingRequestFailed)?;
        let adapters = NetworkAdapters::from_str(adapters_str)?;
        let adapters_md5 = parts
            .next()
            .filter(|path_md5| path_md5.len() == 32)
            .ok_or(AppError::DecodingRequestFailed)?
            .to_string();
        let uninstall_md5 = parts
            .next()
            .filter(|path_md5| path_md5.len() == 32)
            .ok_or(AppError::DecodingRequestFailed)?
            .to_string();
        let disk_signature_md5 = parts
            .next()
            .filter(|path_md5| path_md5.len() == 32)
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
