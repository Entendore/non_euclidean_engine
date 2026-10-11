use super::GeoPoint;

/// Standard Euclidean (flat) geometry — the baseline for comparison.
/// Used as a reference and for transitional/mixed levels.
pub struct EuclideanSpace;

impl EuclideanSpace {
    pub fn new() -> Self {
        Self
    }

    /// Standard Euclidean distance.
    pub fn distance(&self, a: GeoPoint, b: GeoPoint) -> f32 {
        a.distance_to(b)
    }

    /// Simple linear movement.
    pub fn move_point(&self, pos: GeoPoint, dx: f32, dy: f32, speed: f32) -> GeoPoint {
        GeoPoint::new(pos.x + dx * speed, pos.y + dy * speed)
    }

    /// Scale is uniform everywhere in Euclidean space.
    pub fn scale_at(&self, _p: GeoPoint) -> f32 {
        1.0
    }

    /// Triangle angle sum in Euclidean geometry is always exactly 180 degrees.
    pub fn triangle_angle_sum(&self, _a: GeoPoint, _b: GeoPoint, _c: GeoPoint) -> f32 {
        180.0
    }
}