use super::GeoPoint;

/// Internal N-dimensional point for higher-dimensional geometry computations.
/// Stored as a fixed-size array for dimensions 2 through 5.
#[derive(Debug, Clone, Copy)]
pub struct NPoint {
    pub coords: [f32; 5],
    pub dims: usize,
}

impl NPoint {
    pub fn new2(x: f32, y: f32) -> Self {
        Self { coords: [x, y, 0.0, 0.0, 0.0], dims: 2 }
    }

    pub fn new3(x: f32, y: f32, z: f32) -> Self {
        Self { coords: [x, y, z, 0.0, 0.0], dims: 3 }
    }

    pub fn new4(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { coords: [x, y, z, w, 0.0], dims: 4 }
    }

    pub fn new5(x: f32, y: f32, z: f32, w: f32, v: f32) -> Self {
        Self { coords: [x, y, z, w, v], dims: 5 }
    }

    pub fn norm(&self) -> f32 {
        let mut sum = 0.0f32;
        for i in 0..self.dims {
            sum += self.coords[i] * self.coords[i];
        }
        sum.sqrt()
    }

    pub fn norm_sq(&self) -> f32 {
        let mut sum = 0.0f32;
        for i in 0..self.dims {
            sum += self.coords[i] * self.coords[i];
        }
        sum
    }

    pub fn dot(&self, other: &NPoint) -> f32 {
        let mut sum = 0.0f32;
        for i in 0..self.dims.min(other.dims) {
            sum += self.coords[i] * other.coords[i];
        }
        sum
    }

    pub fn distance_to(&self, other: &NPoint) -> f32 {
        let mut sum = 0.0f32;
        let d = self.dims.min(other.dims);
        for i in 0..d {
            let dx = self.coords[i] - other.coords[i];
            sum += dx * dx;
        }
        sum.sqrt()
    }

    /// Multiply all coordinates by a scalar.
    pub fn scaled(&self, s: f32) -> NPoint {
        let mut result = *self;
        for i in 0..self.dims {
            result.coords[i] *= s;
        }
        result
    }

    pub fn normalize(&mut self) {
        let n = self.norm();
        if n > 1e-10 {
            for i in 0..self.dims {
                self.coords[i] /= n;
            }
        }
    }

    pub fn normalized(&self) -> NPoint {
        let mut c = *self;
        c.normalize();
        c
    }

    pub fn lerp(&self, other: &NPoint, t: f32) -> NPoint {
        let mut result = NPoint::new2(0.0, 0.0);
        result.dims = self.dims.max(other.dims);
        for i in 0..result.dims {
            result.coords[i] = self.coords[i] + (other.coords[i] - self.coords[i]) * t;
        }
        result
    }

    /// Slerp (spherical linear interpolation) on the unit sphere in N dimensions.
    pub fn slerp(&self, other: &NPoint, t: f32) -> NPoint {
        let dot = self.dot(other).clamp(-1.0, 1.0);
        let omega = dot.acos();
        if omega < 1e-6 {
            return self.lerp(other, t);
        }
        let sin_omega = omega.sin();
        let a = ((1.0 - t) * omega).sin() / sin_omega;
        let b = (t * omega).sin() / sin_omega;
        let mut result = NPoint::new2(0.0, 0.0);
        result.dims = self.dims.max(other.dims);
        for i in 0..result.dims {
            result.coords[i] = a * self.coords[i] + b * other.coords[i];
        }
        result
    }

    /// Project to 2D GeoPoint using stereographic projection from N dimensions.
    /// The projection plane is the first two coordinates, looking from the "north pole"
    /// in the highest dimension.
    pub fn project_stereographic(&self) -> GeoPoint {
        match self.dims {
            4 => {
                // S³ stereographic: project from (0,0,0,1) to the xyz hyperplane
                let x = self.coords[0];
                let y = self.coords[1];
                let _z = self.coords[2];
                let w = self.coords[3];
                // Project from w=1 pole onto w=0 hyperplane, then take x,y
                let denom = 1.0 - w;
                if denom.abs() < 0.01 {
                    // Near the pole, just return x,y scaled up
                    let scale = 5.0;
                    GeoPoint::new(x * scale, y * scale)
                } else {
                    GeoPoint::new(x / denom, y / denom)
                }
            }
            5 => {
                // S⁴ stereographic: project from (0,0,0,0,1) onto xyzw hyperplane
                let x = self.coords[0];
                let y = self.coords[1];
                let z = self.coords[2];
                let w = self.coords[3];
                let v = self.coords[4];
                let denom = 1.0 - v;
                if denom.abs() < 0.01 {
                    let scale = 5.0;
                    GeoPoint::new(x * scale, y * scale)
                } else {
                    // For 5D, combine x,y with influence from z,w for visual depth
                    let px = x / denom + z * 0.15;
                    let py = y / denom + w * 0.15;
                    GeoPoint::new(px, py)
                }
            }
            _ => GeoPoint::new(self.coords[0], self.coords[1]),
        }
    }

    /// Perspective projection from N-D to 2D.
    /// Uses a virtual camera distance along the last dimension axis.
    pub fn project_perspective(&self, view_dist: f32) -> GeoPoint {
        let d = self.dims;
        // Camera at (0,0,...,view_dist) looking toward origin
        // Perspective: scale = view_dist / (view_dist - last_coord)
        let last_coord = self.coords[d - 1];
        let scale = view_dist / (view_dist - last_coord).max(0.1);

        // Combine higher dimensions as "depth" offset
        let mut offset_x = 0.0f32;
        let mut offset_y = 0.0f32;
        if d >= 4 {
            offset_x += self.coords[2] * 0.3 * scale;
            offset_y += self.coords[3] * 0.3 * scale;
        }
        if d >= 5 {
            offset_x += self.coords[4] * 0.15 * scale;
            offset_y -= self.coords[4] * 0.15 * scale;
        }

        GeoPoint::new(
            self.coords[0] * scale + offset_x,
            self.coords[1] * scale + offset_y,
        )
    }
}

// ============================================================================
// 4D HYPERSPHERICAL GEOMETRY (S³)
// ============================================================================

/// 4D Hyperspherical geometry on the 3-sphere S³.
///
/// The 3-sphere is the surface of a 4D ball. Points are stored as 4D vectors
/// on the unit sphere (|x|²+y²+z²+w² = 1). Movement happens on this surface.
/// The space is finite, closed, and has positive curvature.
///
/// Key property: triangle angle sum > 360° (not just > 180° as in 2D spherical).
/// Parallel transport around a closed loop can rotate a vector by any angle.
///
/// Projection: We use stereographic projection to map S³ → R³, then
/// orthographic projection R³ → R² for display. This creates the
/// characteristic "extra dimension" distortion.
pub struct Hyperspherical4D {
    pub radius: f32,
}

impl Hyperspherical4D {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }

    /// Great-circle distance on S³ (geodesic distance in 4D spherical space).
    /// d = R * arccos(a · b) where a, b are unit 4-vectors.
    pub fn distance(&self, a: &NPoint, b: &NPoint) -> f32 {
        let an = a.normalized();
        let bn = b.normalized();
        let dot = an.dot(&bn).clamp(-1.0, 1.0);
        dot.acos() * self.radius
    }

    /// Move a point on S³ along a direction in the tangent plane.
    /// The direction (dx, dy) is mapped to tangent vectors in 4D space,
    /// and the point is moved along the resulting geodesic.
    pub fn move_point(&self, pos: &NPoint, dx: f32, dy: f32, speed: f32) -> NPoint {
        let normalized = pos.normalized();
        let step = speed * 0.015;

        // Map 2D input to tangent plane at the current position on S³.
        // We use the Gram-Schmidt process to find orthonormal tangent vectors.
        let n = normalized.norm_sq().sqrt();
        if n < 1e-10 { return *pos; }

        // Find two orthonormal tangent vectors at `pos`
        let (t1, t2) = self.tangent_basis(&normalized);

        // Movement vector in tangent plane
        let vx = t1.scaled(dx * step);
        let vy = t2.scaled(dy * step);

        // Move along geodesic: exp_map(pos, direction)
        let mut direction = NPoint::new2(0.0, 0.0);
        direction.dims = 4;
        for i in 0..4 {
            direction.coords[i] = vx.coords[i] + vy.coords[i];
        }
        let dir_norm = direction.norm();
        if dir_norm < 1e-10 { return *pos; }

        // Exponential map: rotate pos toward direction by angle = dir_norm
        let angle = dir_norm.min(0.1); // Clamp for stability
        let dir_normed = direction.normalized();

        // Rodrigues-like rotation in 4D:
        // new_pos = pos * cos(angle) + dir * sin(angle)
        let mut result = NPoint::new2(0.0, 0.0);
        result.dims = 4;
        for i in 0..4 {
            result.coords[i] = normalized.coords[i] * angle.cos()
                + dir_normed.coords[i] * angle.sin();
        }
        // Renormalize to stay on S³
        result.normalize();
        result
    }

    /// Compute two orthonormal tangent vectors at a point on S³.
    fn tangent_basis(&self, pos: &NPoint) -> (NPoint, NPoint) {
        // Gram-Schmidt: find vectors perpendicular to pos
        let mut t1 = NPoint::new2(0.0, 0.0);
        t1.dims = 4;
        // Choose a reference vector not parallel to pos
        t1.coords[0] = 1.0;
        // Project out the pos component
        let dot = t1.dot(pos);
        for i in 0..4 {
            t1.coords[i] -= dot * pos.coords[i];
        }
        let t1n = t1.norm();
        if t1n < 1e-6 {
            // pos is nearly (1,0,0,0), try y-axis instead
            t1.coords[0] = 0.0;
            t1.coords[1] = 1.0;
            let dot = t1.dot(pos);
            for i in 0..4 {
                t1.coords[i] -= dot * pos.coords[i];
            }
        }
        t1.normalize();

        // t2 = pos × t1 (generalized cross product in 4D)
        // In 4D, the "cross product" of 2 vectors gives 2 vectors.
        // We compute t2 as a vector perpendicular to both pos and t1.
        let mut t2 = NPoint::new2(0.0, 0.0);
        t2.dims = 4;
        // Use the Hodge dual / simple perpendicular construction
        t2.coords[0] = pos.coords[1] * t1.coords[2] - pos.coords[2] * t1.coords[1];
        t2.coords[1] = pos.coords[2] * t1.coords[0] - pos.coords[0] * t1.coords[2];
        t2.coords[2] = pos.coords[0] * t1.coords[1] - pos.coords[1] * t1.coords[0];
        t2.coords[3] = pos.coords[3]; // Add w-component variation

        let t2n = t2.norm();
        if t2n < 1e-6 {
            t2.coords[0] = pos.coords[3] * t1.coords[0] - pos.coords[0] * t1.coords[3];
            t2.coords[1] = pos.coords[3] * t1.coords[1] - pos.coords[1] * t1.coords[3];
            t2.coords[2] = pos.coords[3] * t1.coords[2] - pos.coords[2] * t1.coords[3];
            t2.coords[3] = 0.0;
        }
        t2.normalize();
        (t1, t2)
    }

    /// Get the local scale factor for rendering at a given point.
    /// In the stereographic projection, areas near the projection pole appear larger.
    pub fn scale_at(&self, pos: &NPoint) -> f32 {
        let w = pos.coords[3];
        let denom = (1.0 - w).abs().max(0.05);
        1.0 / (denom * denom)
    }

    /// Compute the volume of a tetrahedron on S³ (4D analog of triangle area).
    /// Uses the spherical excess formula generalized to 4D.
    pub fn simplex_excess(&self, a: &NPoint, b: &NPoint, c: &NPoint, d: &NPoint) -> f32 {
        let ab = self.distance(a, b);
        let ac = self.distance(a, c);
        let ad = self.distance(a, d);
        let bc = self.distance(b, c);
        let bd = self.distance(b, d);
        let cd = self.distance(c, d);

        // Approximate 4D spherical excess using dihedral angle sum
        // In 4D, the "excess" relates to the volume of the spherical simplex
        let perimeters = ab + ac + bc + ad + bd + cd;
        let r2 = self.radius * self.radius;
        // Girard-like generalization for 4D
        (perimeters / (2.0 * std::f32::consts::PI * self.radius) - 1.0) * r2
    }

    /// Check if movement wrapped around (teleported across the hypersphere).
    pub fn did_wrap(&self, old: &NPoint, new: &NPoint) -> bool {
        let dot = old.normalized().dot(&new.normalized());
        dot < 0.5 // More than 60 degrees of movement = likely a wrap
    }

    /// Get the 2D projected position for rendering.
    pub fn to_2d(&self, pos: &NPoint) -> GeoPoint {
        pos.project_stereographic()
    }

    /// Triangle angle sum on S³ (can exceed 360° due to extra curvature dimension).
    pub fn triangle_angle_sum(&self, a: &NPoint, b: &NPoint, c: &NPoint) -> f32 {
        let ab = self.distance(a, b);
        let bc = self.distance(b, c);
        let ca = self.distance(c, a);

        // Spherical law of cosines for S³
        let cos_a = (bc.cos() - ab.cos() * ca.cos()) / (ab.sin().max(1e-6) * ca.sin().max(1e-6));
        let cos_b = (ca.cos() - ab.cos() * bc.cos()) / (ab.sin().max(1e-6) * bc.sin().max(1e-6));
        let cos_c = (ab.cos() - bc.cos() * ca.cos()) / (bc.sin().max(1e-6) * ca.sin().max(1e-6));

        let angle_a = cos_a.clamp(-1.0, 1.0).acos().to_degrees();
        let angle_b = cos_b.clamp(-1.0, 1.0).acos().to_degrees();
        let angle_c = cos_c.clamp(-1.0, 1.0).acos().to_degrees();

        angle_a + angle_b + angle_c
    }
}

// ============================================================================
// 5D HYPERSPHERICAL GEOMETRY (S⁴)
// ============================================================================

/// 5D Hyperspherical geometry on the 4-sphere S⁴.
///
/// The 4-sphere is the surface of a 5D ball. Points are 5D unit vectors.
/// This space has even stronger positive curvature than S³.
/// Triangle angle sums can be much larger (up to ~540°).
///
/// Key property: In 5D spherical geometry, you can have up to 5 mutually
/// perpendicular geodesics all meeting at a single point, and the space
/// has "more room" in higher dimensions than intuition suggests.
pub struct Hyperspherical5D {
    pub radius: f32,
}

impl Hyperspherical5D {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }

    /// Great-circle distance on S⁴.
    pub fn distance(&self, a: &NPoint, b: &NPoint) -> f32 {
        let an = a.normalized();
        let bn = b.normalized();
        let dot = an.dot(&bn).clamp(-1.0, 1.0);
        dot.acos() * self.radius
    }

    /// Move a point on S⁴ using the exponential map.
    pub fn move_point(&self, pos: &NPoint, dx: f32, dy: f32, speed: f32) -> NPoint {
        let normalized = pos.normalized();
        let step = speed * 0.012;

        let (t1, t2) = self.tangent_basis(&normalized);

        let vx = t1.scaled(dx * step);
        let vy = t2.scaled(dy * step);

        let mut direction = NPoint::new2(0.0, 0.0);
        direction.dims = 5;
        for i in 0..5 {
            direction.coords[i] = vx.coords[i] + vy.coords[i];
        }
        let dir_norm = direction.norm();
        if dir_norm < 1e-10 { return *pos; }

        let angle = dir_norm.min(0.08);
        let dir_normed = direction.normalized();

        let mut result = NPoint::new2(0.0, 0.0);
        result.dims = 5;
        for i in 0..5 {
            result.coords[i] = normalized.coords[i] * angle.cos()
                + dir_normed.coords[i] * angle.sin();
        }
        result.normalize();
        result
    }

    fn tangent_basis(&self, pos: &NPoint) -> (NPoint, NPoint) {
        let mut t1 = NPoint::new2(0.0, 0.0);
        t1.dims = 5;
        t1.coords[0] = 1.0;
        let dot = t1.dot(pos);
        for i in 0..5 {
            t1.coords[i] -= dot * pos.coords[i];
        }
        let t1n = t1.norm();
        if t1n < 1e-6 {
            t1.coords[0] = 0.0;
            t1.coords[1] = 1.0;
            let dot = t1.dot(pos);
            for i in 0..5 {
                t1.coords[i] -= dot * pos.coords[i];
            }
        }
        t1.normalize();

        let mut t2 = NPoint::new2(0.0, 0.0);
        t2.dims = 5;
        t2.coords[0] = pos.coords[1] * t1.coords[2] - pos.coords[2] * t1.coords[1];
        t2.coords[1] = pos.coords[2] * t1.coords[3] - pos.coords[3] * t1.coords[2];
        t2.coords[2] = pos.coords[3] * t1.coords[4] - pos.coords[4] * t1.coords[3];
        t2.coords[3] = pos.coords[4] * t1.coords[0] - pos.coords[0] * t1.coords[4];
        t2.coords[4] = pos.coords[0] * t1.coords[1] - pos.coords[1] * t1.coords[0];

        let t2n = t2.norm();
        if t2n < 1e-6 {
            t2.coords[0] = 1.0;
            for i in 0..5 { t2.coords[i] = 0.0; }
            t2.coords[2] = 1.0;
            let dot = t2.dot(pos);
            for i in 0..5 { t2.coords[i] -= dot * pos.coords[i]; }
            let dot2 = t2.dot(&t1);
            for i in 0..5 { t2.coords[i] -= dot2 * t1.coords[i]; }
        }
        t2.normalize();
        (t1, t2)
    }

    /// Scale factor for 5D stereographic projection.
    pub fn scale_at(&self, pos: &NPoint) -> f32 {
        let v = pos.coords[4];
        let denom = (1.0 - v).abs().max(0.05);
        1.0 / (denom * denom * denom.sqrt()) // Extra compression for 5D
    }

    /// Project 5D point to 2D for rendering.
    pub fn to_2d(&self, pos: &NPoint) -> GeoPoint {
        pos.project_stereographic()
    }

    pub fn did_wrap(&self, old: &NPoint, new: &NPoint) -> bool {
        let dot = old.normalized().dot(&new.normalized());
        dot < 0.5
    }

    pub fn triangle_angle_sum(&self, a: &NPoint, b: &NPoint, c: &NPoint) -> f32 {
        let ab = self.distance(a, b);
        let bc = self.distance(b, c);
        let ca = self.distance(c, a);

        let cos_a = (bc.cos() - ab.cos() * ca.cos()) / (ab.sin().max(1e-6) * ca.sin().max(1e-6));
        let cos_b = (ca.cos() - ab.cos() * bc.cos()) / (ab.sin().max(1e-6) * bc.sin().max(1e-6));
        let cos_c = (ab.cos() - bc.cos() * ca.cos()) / (bc.sin().max(1e-6) * ca.sin().max(1e-6));

        let angle_a = cos_a.clamp(-1.0, 1.0).acos().to_degrees();
        let angle_b = cos_b.clamp(-1.0, 1.0).acos().to_degrees();
        let angle_c = cos_c.clamp(-1.0, 1.0).acos().to_degrees();

        angle_a + angle_b + angle_c
    }
}

// ============================================================================
// 4D HYPERBOLIC GEOMETRY (H⁴ - Poincaré Ball)
// ============================================================================

/// 4D Hyperbolic geometry using the Poincaré ball model.
///
/// The entire 4D hyperbolic space is represented inside a 4D unit ball.
/// Geodesics are circular arcs (or straight lines through the origin).
/// The metric is: ds² = 4/(1-|x|²)² * Σdxᵢ²
///
/// Key property: More "room" near the boundary than 2D hyperbolic.
/// Volume grows exponentially faster with radius in 4D vs 2D.
/// A 4D hyperbolic tiling of {5,3,3,3} has dodecahedral cells.
pub struct Hyperbolic4D {
    pub curvature: f32,
}

impl Hyperbolic4D {
    pub fn new(curvature: f32) -> Self {
        Self { curvature }
    }

    /// 4D hyperbolic distance in the Poincaré ball model.
    /// dist = acosh(1 + 2|a-b|²/((1-|a|²)(1-|b|²)))
    pub fn distance(&self, a: &NPoint, b: &NPoint) -> f32 {
        let a2 = a.norm_sq();
        let b2 = b.norm_sq();
        if a2 >= 1.0 || b2 >= 1.0 { return f32::MAX; }

        let d2 = a.distance_to(b).powi(2);
        let den = (1.0 - a2) * (1.0 - b2);
        if den <= 0.0 { return f32::MAX; }

        let arg = 1.0 + 2.0 * d2 / den;
        if arg <= 1.0 { return 0.0; }
        arg.ln().sqrt() * self.curvature.abs().sqrt().recip()
    }

    /// Move a point in 4D hyperbolic space.
    /// Uses the conformal factor (1-|p|²)/2 for each dimension.
    pub fn move_point(&self, pos: &NPoint, dx: f32, dy: f32, speed: f32) -> NPoint {
        let r2 = pos.norm_sq();
        if r2 >= 0.98 { return *pos; }

        // 4D conformal factor
        let conformal = (1.0 - r2) * speed * 0.5;
        let step = speed * 0.003;

        let (t1, t2) = self.tangent_basis(pos);

        let mut result = *pos;
        for i in 0..4 {
            result.coords[i] += (dx * t1.coords[i] + dy * t2.coords[i]) * step * conformal;
        }

        // Clamp to ball
        let new_r2 = result.norm_sq();
        if new_r2 >= 0.97 {
            let scale = 0.96 / new_r2.sqrt();
            for i in 0..4 { result.coords[i] *= scale; }
        }
        result
    }

    fn tangent_basis(&self, pos: &NPoint) -> (NPoint, NPoint) {
        let mut t1 = NPoint::new2(1.0, 0.0);
        t1.dims = 4;
        let dot = t1.dot(pos);
        for i in 0..4 { t1.coords[i] -= dot * pos.coords[i]; }
        t1.normalize();

        let mut t2 = NPoint::new2(0.0, 1.0);
        t2.dims = 4;
        let dot = t2.dot(pos);
        for i in 0..4 { t2.coords[i] -= dot * pos.coords[i]; }
        let dot2 = t2.dot(&t1);
        for i in 0..4 { t2.coords[i] -= dot2 * t1.coords[i]; }
        t2.normalize();
        (t1, t2)
    }

    /// 4D conformal scale factor.
    pub fn scale_at(&self, pos: &NPoint) -> f32 {
        let r2 = pos.norm_sq();
        if r2 >= 1.0 { return 0.001; }
        (1.0 - r2) / 2.0
    }

    /// Project 4D point to 2D for rendering.
    pub fn to_2d(&self, pos: &NPoint) -> GeoPoint {
        pos.project_perspective(3.0)
    }

    pub fn is_inside(&self, pos: &NPoint) -> bool {
        pos.norm_sq() < 1.0
    }
}

// ============================================================================
// 5D HYPERBOLIC GEOMETRY (H⁵ - Poincaré Ball)
// ============================================================================

/// 5D Hyperbolic geometry using the Poincaré ball model.
///
/// Even more extreme version of 4D hyperbolic space.
/// Volume grows as ~exp(4r) compared to ~exp(r) in 2D hyperbolic.
/// This means the space near the boundary is incomprehensibly vast.
///
/// For a game, we represent the player position as a 5D point inside
/// the unit 5-ball, and project to 2D for display.
pub struct Hyperbolic5D {
    pub curvature: f32,
}

impl Hyperbolic5D {
    pub fn new(curvature: f32) -> Self {
        Self { curvature }
    }

    /// 5D hyperbolic distance in the Poincaré ball model.
    pub fn distance(&self, a: &NPoint, b: &NPoint) -> f32 {
        let a2 = a.norm_sq();
        let b2 = b.norm_sq();
        if a2 >= 1.0 || b2 >= 1.0 { return f32::MAX; }

        let d2 = a.distance_to(b).powi(2);
        let den = (1.0 - a2) * (1.0 - b2);
        if den <= 0.0 { return f32::MAX; }

        let arg = 1.0 + 2.0 * d2 / den;
        if arg <= 1.0 { return 0.0; }
        arg.ln().sqrt() * self.curvature.abs().sqrt().recip()
    }

    /// Move a point in 5D hyperbolic space.
    pub fn move_point(&self, pos: &NPoint, dx: f32, dy: f32, speed: f32) -> NPoint {
        let r2 = pos.norm_sq();
        if r2 >= 0.98 { return *pos; }

        let conformal = (1.0 - r2) * speed * 0.4;
        let step = speed * 0.002;

        let (t1, t2) = self.tangent_basis(pos);

        let mut result = *pos;
        for i in 0..5 {
            result.coords[i] += (dx * t1.coords[i] + dy * t2.coords[i]) * step * conformal;
        }

        let new_r2 = result.norm_sq();
        if new_r2 >= 0.97 {
            let scale = 0.96 / new_r2.sqrt();
            for i in 0..5 { result.coords[i] *= scale; }
        }
        result
    }

    fn tangent_basis(&self, pos: &NPoint) -> (NPoint, NPoint) {
        let mut t1 = NPoint::new2(1.0, 0.0);
        t1.dims = 5;
        let dot = t1.dot(pos);
        for i in 0..5 { t1.coords[i] -= dot * pos.coords[i]; }
        t1.normalize();

        let mut t2 = NPoint::new2(0.0, 1.0);
        t2.dims = 5;
        let dot = t2.dot(pos);
        for i in 0..5 { t2.coords[i] -= dot * pos.coords[i]; }
        let dot2 = t2.dot(&t1);
        for i in 0..5 { t2.coords[i] -= dot2 * t1.coords[i]; }
        t2.normalize();
        (t1, t2)
    }

    /// 5D conformal scale factor (more extreme than 4D).
    pub fn scale_at(&self, pos: &NPoint) -> f32 {
        let r2 = pos.norm_sq();
        if r2 >= 1.0 { return 0.001; }
        let base = (1.0 - r2) / 2.0;
        base * base.sqrt() // Extra falloff for 5D
    }

    /// Project 5D point to 2D for rendering.
    pub fn to_2d(&self, pos: &NPoint) -> GeoPoint {
        pos.project_perspective(4.0)
    }

    pub fn is_inside(&self, pos: &NPoint) -> bool {
        pos.norm_sq() < 1.0
    }
}

// ============================================================================
// HYPERBOLIC GEOMETRY ENHANCEMENTS (improved 2D)
// ============================================================================

/// Enhanced Poincaré disk functions with proper geodesic arc computation.

/// Compute the center and radius of the geodesic arc circle in the Poincaré disk.
/// Returns (center_x, center_y, radius) of the arc circle, or None if it's
/// a diameter (straight line through origin).
pub fn geodesic_arc_circle(ax: f32, ay: f32, bx: f32, by: f32) -> Option<(f32, f32, f32)> {
    let dx = bx - ax;
    let dy = by - ay;
    let cross = ay * dx - ax * dy;

    if cross.abs() < 1e-6 {
        return None; // Diameter line
    }

    let mx = (ax + bx) / 2.0;
    let my = (ay + by) / 2.0;
    let t = (1.0 - ax * bx - ay * by) / (2.0 * cross);

    let cx = mx - t * dy;
    let cy = my + t * dx;
    let r_sq = cx * cx + cy * cy - 1.0;
    if r_sq <= 0.001 { return None; }
    let r = r_sq.sqrt();

    Some((cx, cy, r))
}

/// Compute points along a proper hyperbolic geodesic arc between two points.
/// Unlike linear interpolation, this follows the circular arc that represents
/// the true shortest path in the Poincaré disk.
pub fn geodesic_arc_points(ax: f32, ay: f32, bx: f32, by: f32, segments: usize) -> Vec<(f32, f32)> {
    match geodesic_arc_circle(ax, ay, bx, by) {
        None => {
            // Straight line (diameter)
            (0..=segments).map(|i| {
                let t = i as f32 / segments as f32;
                (ax + (bx - ax) * t, ay + (by - ay) * t)
            }).collect()
        }
        Some((cx, cy, r)) => {
            let start_angle = (ay - cy).atan2(ax - cx);
            let end_angle = (by - cy).atan2(bx - cx);

            // Determine which arc stays inside the disk
            let mid_angle = (start_angle + end_angle) / 2.0;
            let mid_alt = mid_angle + std::f32::consts::PI;

            let d1 = (cx + r * mid_angle.cos()).powi(2) + (cy + r * mid_angle.sin()).powi(2);
            let d2 = (cx + r * mid_alt.cos()).powi(2) + (cy + r * mid_alt.sin()).powi(2);

            let (sa, ea, forward) = if d1 < d2 {
                (start_angle, end_angle, true)
            } else {
                (end_angle, start_angle, false)
            };

            // Compute angular span
            let span = if forward {
                if ea >= sa { ea - sa } else { ea - sa + std::f32::consts::TAU }
            } else {
                if sa >= ea { sa - ea } else { sa - ea + std::f32::consts::TAU }
            };

            (0..=segments).map(|i| {
                let t = i as f32 / segments as f32;
                let angle = if forward {
                    sa + span * t
                } else {
                    sa - span * t
                };
                (cx + r * angle.cos(), cy + r * angle.sin())
            }).collect()
        }
    }
}

/// Enhanced conformal factor that includes curvature-dependent distortion.
/// Returns a multiplier for rendering size at a given point in the Poincaré disk.
pub fn enhanced_conformal_factor(x: f32, y: f32, base_curvature: f32) -> f32 {
    let r2 = x * x + y * y;
    if r2 >= 1.0 { return 0.001; }
    let basic = (1.0 - r2) / 2.0;
    // Apply curvature: more negative curvature = more distortion
    let curvature_mult = base_curvature.abs().sqrt().max(0.5);
    basic * curvature_mult
}