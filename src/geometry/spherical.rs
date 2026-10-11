use super::GeoPoint;

/// Spherical geometry on a sphere of given radius.
/// Points are stored as (x, y) on a Mercator-like projection or
/// as (latitude, longitude) mapped to [-1, 1] range.
/// The space is finite but "wraps around" — going far enough in any direction
/// brings you back to where you started.
pub struct SphericalSpace {
    pub radius: f32, // sphere radius, controls curvature
}

impl SphericalSpace {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }

    /// Convert (x, y) in [-1, 1] to spherical coordinates (lat, lon) in radians.
    fn to_lat_lon(&self, p: GeoPoint) -> (f32, f32) {
        let lat = p.y * std::f32::consts::FRAC_PI_2; // [-pi/2, pi/2]
        let lon = p.x * std::f32::consts::PI; // [-pi, pi]
        (lat, lon)
    }

    /// Spherical distance between two points using the haversine formula.
    pub fn distance(&self, a: GeoPoint, b: GeoPoint) -> f32 {
        let (lat1, lon1) = self.to_lat_lon(a);
        let (lat2, lon2) = self.to_lat_lon(b);

        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;

        let haversine_lat = (dlat / 2.0).sin().powi(2);
        let haversine_lon = (dlon / 2.0).sin().powi(2) * lat1.cos() * lat2.cos();

        let h = haversine_lat + haversine_lon;
        let great_circle_dist = 2.0 * h.sqrt().asin();

        great_circle_dist * self.radius
    }

    /// Move a point on the sphere surface by a direction and speed.
    /// Takes into account that moving "east" is faster at the equator than near poles.
    pub fn move_point(&self, pos: GeoPoint, dx: f32, dy: f32, speed: f32) -> GeoPoint {
        let (lat, lon) = self.to_lat_lon(pos);

        // East-west movement is scaled by cos(latitude)
        let cos_lat = lat.cos().max(0.01);

        let new_lon = lon + dx * speed * 0.02 / cos_lat;
        let new_lat = lat + dy * speed * 0.02;

        // Wrap longitude to [-pi, pi]
        let mut wrapped_lon = new_lon;
        while wrapped_lon > std::f32::consts::PI {
            wrapped_lon -= 2.0 * std::f32::consts::PI;
        }
        while wrapped_lon < -std::f32::consts::PI {
            wrapped_lon += 2.0 * std::f32::consts::PI;
        }

        // Clamp latitude to avoid pole singularity
        let clamped_lat = new_lat.clamp(
            -std::f32::consts::FRAC_PI_2 * 0.95,
            std::f32::consts::FRAC_PI_2 * 0.95,
        );

        // Convert back to [-1, 1]
        GeoPoint::new(
            wrapped_lon / std::f32::consts::PI,
            clamped_lat / std::f32::consts::FRAC_PI_2,
        )
    }

    /// Check if a point has "wrapped around" (teleported) during movement.
    /// Returns true if the displacement is large, indicating a wrap.
    pub fn did_wrap(&self, old: GeoPoint, new: GeoPoint) -> bool {
        (old.x - new.x).abs() > 0.5
    }

    /// Get the scale factor at a point (Mercator distortion).
    /// Near the poles, things appear stretched horizontally.
    pub fn scale_at(&self, p: GeoPoint) -> f32 {
        let (lat, _) = self.to_lat_lon(p);
        lat.cos().max(0.1)
    }

    /// Compute the angle sum of a triangle on the sphere.
    /// In spherical geometry, the angle sum is always GREATER than 180 degrees.
    /// The excess gives the area of the triangle (Girard's theorem).
    pub fn triangle_angle_sum(&self, a: GeoPoint, b: GeoPoint, c: GeoPoint) -> f32 {
        // Convert to 3D Cartesian on the sphere
        let va = self.to_3d(a);
        let vb = self.to_3d(b);
        let vc = self.to_3d(c);

        let angle_a = angle_between(&subtract(&vb, &va), &subtract(&vc, &va));
        let angle_b = angle_between(&subtract(&va, &vb), &subtract(&vc, &vb));
        let angle_c = angle_between(&subtract(&va, &vc), &subtract(&vb, &vc));

        (angle_a + angle_b + angle_c).to_degrees()
    }

    /// Convert a 2D point to 3D coordinates on the sphere surface.
    pub fn to_3d(&self, p: GeoPoint) -> [f32; 3] {
        let (lat, lon) = self.to_lat_lon(p);
        [
            self.radius * lat.cos() * lon.cos(),
            self.radius * lat.cos() * lon.sin(),
            self.radius * lat.sin(),
        ]
    }
}

fn subtract(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn angle_between(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let mag_a = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    let mag_b = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    if mag_a < 1e-10 || mag_b < 1e-10 {
        return 0.0;
    }
    (dot / (mag_a * mag_b)).clamp(-1.0, 1.0).acos()
}