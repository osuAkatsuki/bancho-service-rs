pub const fn clamp_lat(lat: f32) -> f32 {
    const MAX_DEGREES: f32 = 90.0;
    const MIN_DEGREES: f32 = -90.0;
    if lat > MAX_DEGREES {
        let adt = lat - MAX_DEGREES;
        MIN_DEGREES + adt
    } else if lat < MIN_DEGREES {
        let adt = lat - MIN_DEGREES;
        MAX_DEGREES + adt
    } else {
        lat
    }
}

pub const fn clamp_lon(lon: f32) -> f32 {
    const MAX_DEGREES: f32 = 180.0;
    const MIN_DEGREES: f32 = -180.0;
    if lon > MAX_DEGREES {
        let adt = lon - MAX_DEGREES;
        MIN_DEGREES + adt
    } else if lon < MIN_DEGREES {
        let adt = lon - MIN_DEGREES;
        MAX_DEGREES + adt
    } else {
        lon
    }
}

const EARTH_CIRCUMFERENCE_KM: f32 = 40075.017;
const KM_PER_DEGREE: f32 = EARTH_CIRCUMFERENCE_KM / 360.0;
const DEGREE_PER_KM: f32 = 1.0 / KM_PER_DEGREE;

const fn to_radians(angle: f32) -> f32 {
    (angle / 180.0) * std::f32::consts::PI
}

pub fn displace_location(lat: f32, lon: f32, max_distance_km: f32) -> (f32, f32) {
    let min_distance_km = max_distance_km / 4.0;
    let distance_km = rand::random_range(min_distance_km..=max_distance_km);
    let angle_deg = rand::random_range(0.0..360.0);
    let angle_rad = to_radians(angle_deg);
    let lat_displacement = distance_km * angle_rad.sin();
    let lon_displacement = distance_km * angle_rad.cos();
    let lat_displacement_degrees = lat_displacement * DEGREE_PER_KM;
    let lon_displacement_degrees = lon_displacement * DEGREE_PER_KM;

    let displaced_lat = clamp_lat(lat + lat_displacement_degrees);
    let displaced_lon = clamp_lon(lon + lon_displacement_degrees);
    (displaced_lat, displaced_lon)
}
