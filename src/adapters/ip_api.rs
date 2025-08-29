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
    pub country_code: Option<String>,
    #[serde(rename = "lat")]
    pub latitude: Option<f32>,
    #[serde(rename = "lon")]
    pub longitude: Option<f32>,
}

pub async fn get_ip_info(ip: IpAddr) -> ServiceResult<IPLocation> {
    let url = match ip.is_loopback() {
        true => make_url(""),
        false => make_url(ip),
    };
    let location: IPLocation = reqwest::get(url).await?.json().await?;
    Ok(location)
}
