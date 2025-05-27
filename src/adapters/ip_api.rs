use crate::common::error::ServiceResult;
use serde::Deserialize;
use std::fmt::Display;
use std::net::IpAddr;

fn make_url<T: Display>(ip: T) -> String {
    format!("http://ip-api.com/json/{ip}?fields=countryCode,lat,lon")
}

#[derive(Debug, Deserialize)]
pub struct IPLocation {
    #[serde(rename = "countryCode")]
    pub country_code: String,
    #[serde(rename = "lat")]
    pub latitude: f32,
    #[serde(rename = "lon")]
    pub longitude: f32,
}

pub async fn get_ip_info(ip: IpAddr) -> ServiceResult<IPLocation> {
    let url = match ip.is_loopback() {
        true => make_url(""),
        false => make_url(ip),
    };
    let location: IPLocation = reqwest::get(url).await?.json().await?;
    Ok(location)
}
