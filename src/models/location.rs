use crate::common::location;
use bancho_protocol::structures::Country;

pub struct LocationInformation {
    pub country: Country,
    pub latitude: f32,
    pub longitude: f32,
}

impl LocationInformation {
    pub fn offset_randomly(mut self, show_exact: bool) -> Self {
        let max_offset_by_km = match show_exact {
            true => 1.0,
            false => 20.0,
        };
        let (lat, lon) =
            location::displace_location(self.latitude, self.longitude, max_offset_by_km);
        self.latitude = lat;
        self.longitude = lon;
        self
    }
}
