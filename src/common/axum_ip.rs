use crate::common::error::ServiceResult;
use crate::models::bancho::BanchoResponse;
use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

pub struct IpAddrInfo {
    pub ip_addr: IpAddr,
}

async fn get_ip_addr(parts: &mut Parts) -> ServiceResult<IpAddrInfo> {
    if let Some(ip) = parts.headers.get("CF-Connecting-IP") {
        let ip_addr = IpAddr::from_str(ip.to_str()?)?;
        Ok(IpAddrInfo { ip_addr })
    } else if let Some(ip) = parts.headers.get("X-Forwarded-For") {
        let ip_addr = IpAddr::from_str(ip.to_str()?)?;
        Ok(IpAddrInfo { ip_addr })
    } else {
        let info = <ConnectInfo<SocketAddr>>::from_request_parts(parts, &()).await?;
        let ip_addr = info.ip();
        Ok(IpAddrInfo { ip_addr })
    }
}

impl<S: Sync + Send> FromRequestParts<S> for IpAddrInfo {
    type Rejection = BanchoResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match get_ip_addr(parts).await {
            Ok(ip_addr) => Ok(ip_addr),
            Err(e) => Err(BanchoResponse::error(None, e)),
        }
    }
}
