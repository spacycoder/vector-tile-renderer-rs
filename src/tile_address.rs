#[derive(Debug)]
pub struct TileAddress {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl TileAddress {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        TileAddress { x, y, z }
    }

    pub fn get_tiles(&self, radius: u32) -> Vec<TileAddress> {
        let min_x = self.x - radius;
        let max_x = self.x + radius;
        let min_y = self.y - radius;
        let max_y = self.y + radius;

        let mut tiles = vec![];
        for x in min_x..max_x + 1 {
            for y in min_y..max_y + 1 {
                tiles.push(TileAddress::new(x, y, self.z));
            }
        }

        return tiles;
    }
}

pub const EARTH_RADIUS_METER: f64 = 6378137.0;
pub const EARTH_CIRCUMFERENCE_METERS: f64 = EARTH_RADIUS_METER * std::f64::consts::PI * 2.0;
pub const EARTH_HALF_CIRCUMFERENCE_METERS: f64 = EARTH_RADIUS_METER * std::f64::consts::PI;

pub fn latlon_to_tile_address(latitude: f64, longitude: f64, zoom: u32) -> TileAddress {
    let meters_per_tile = EARTH_CIRCUMFERENCE_METERS / (1 << zoom) as f64;

    let pi = std::f64::consts::PI;

    let x = longitude * EARTH_HALF_CIRCUMFERENCE_METERS / 180.0;
    let y = (0.25 * pi + latitude * pi / 360.0).tan().ln() * EARTH_RADIUS_METER;

    let tile_x = ((x + EARTH_HALF_CIRCUMFERENCE_METERS) / meters_per_tile) as u32;
    let tile_y = ((EARTH_HALF_CIRCUMFERENCE_METERS - y) / meters_per_tile) as u32;
    return TileAddress::new(tile_x, tile_y, zoom);
}
