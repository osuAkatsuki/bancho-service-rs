use crate::adapters::ip_api;
use crate::models::location::LocationInformation;
use bancho_protocol::structures::Country;
use std::net::IpAddr;
use tracing::warn;

pub async fn get_location(
    ip_address: IpAddr,
    user_country: Country,
    show_exact: bool,
) -> LocationInformation {
    match ip_api::get_ip_info(ip_address).await {
        Ok(location) => {
            let country = location.country_code.map_or(user_country, |code| {
                Country::try_from_iso3166_2(&code).unwrap_or(user_country)
            });
            let location = LocationInformation {
                country,
                latitude: location.latitude.unwrap_or(0.0),
                longitude: location.longitude.unwrap_or(0.0),
            };
            location.offset_randomly(show_exact)
        }
        Err(e) => {
            warn!(
                ip_address = ip_address.to_string(),
                "Failed getting location for IP address: {e:?}"
            );
            LocationInformation {
                country: user_country,
                latitude: 0.0,
                longitude: 0.0,
            }
        }
    }
}
