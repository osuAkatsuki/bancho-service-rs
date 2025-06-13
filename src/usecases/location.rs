use crate::adapters::ip_api;
use crate::models::location::LocationInformation;
use bancho_protocol::structures::Country;
use std::net::IpAddr;
use tracing::error;

pub async fn get_location(
    ip_address: IpAddr,
    user_country: Country,
    show_exact: bool,
) -> LocationInformation {
    match ip_api::get_ip_info(ip_address).await {
        Ok(location) => {
            let country =
                Country::try_from_iso3166_2(&location.country_code).unwrap_or(user_country);
            let location = LocationInformation {
                country,
                latitude: location.latitude,
                longitude: location.longitude,
            };
            location.offset_randomly(show_exact)
        }
        Err(e) => {
            error!("Error getting location for session: {e:?}");
            LocationInformation {
                country: user_country,
                latitude: 0.0,
                longitude: 0.0,
            }
        }
    }
}
