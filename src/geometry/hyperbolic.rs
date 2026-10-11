use super::GeoPoint;
use super::hyperspace::{geodesic_arc_points, enhanced_conformal_factor};

/// Poincaré disk model of hyperbolic geometry.
/// The entire hyperbolic plane is represented inside the unit disk (radius < 1).
/// Geodesics appear as circular arcs perpendicular to the boundary circle.
/// Key property: the disk appears finite but contains infinite space.
pub struct HyperbolicSpace {
    pub curvature: f32,
}

impl HyperbolicSpace {
    pub fn new(curvature: f32) -> Self {
        Self { curvature }
    }

    /// Compute the hyperbolic distance between two points in the Poincaré disk.
    /// Uses the metric: dist = acosh(1 + 2*|p-q|^2 / ((1-|p|^2)(1-|q|^2)))
    pub fn distance(&self, a: GeoPoint, b: GeoPoint) -> f32 {
        let axy = a.x * a.x + a.y * a.y;
        let bxy = b.x * b.x + b.y * b.y;
        if axy >= 1.0 || bxy >= 1.0 {
            return f32::MAX;
        }
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let num = dx * dx + dy * dy;
        let den = (1.0 - axy) * (1.0 - bxy);
        if den <= 0.0 {
            return f32::MAX;
        }
        let arg = 1.0 + 2.0 * num / den;
        if arg <= 1.0 {
            return 0.0;
        }
        arg.ln().sqrt() * self.curvature.abs().sqrt().recip()
    }

    /// Möbius transformation: translate point `p` so that `center` moves to origin.
    /// This is how "moving" the player works in hyperbolic space.
    pub fn mobius_translate(&self, p: GeoPoint, center: GeoPoint) -> GeoPoint {
        let c2 = center.x * center.x + center.y * center.y;
        if c2 >= 1.0 {
            return p;
        }
        let px = p.x - center.x;
        let py = p.y - center.y;
        let denom = 1.0 - center.x * p.x - center.y * p.y;
        if denom.abs() < 1e-10 {
            return GeoPoint::new(0.0, 0.0);
        }
        GeoPoint::new(
            (px - c2 * p.x + center.y * (p.x * center.y - p.y * center.x)) / denom,
            (py - c2 * p.y + center.x * (p.y * center.x - p.x * center.y)) / denom,
        )
    }

    /// Move a point by a small step in a given direction (dx, dy) in hyperbolic space.
    pub fn move_point(&self, pos: GeoPoint, dx: f32, dy: f32, speed: f32) -> GeoPoint {
        let scale = (1.0 - (pos.x * pos.x + pos.y * pos.y)) * speed;
        let mut nx = pos.x + dx * scale;
        let mut ny = pos.y + dy * scale;
        // Clamp to stay inside the unit disk
        let r2 = nx * nx + ny * ny;
        if r2 >= 0.98 {
            let r = r2.sqrt();
            nx *= 0.97 / r;
            ny *= 0.97 / r;
        }
        GeoPoint::new(nx, ny)
    }

    /// Get the hyperbolic "zoom" factor at a given point (objects near edge appear smaller).
    /// Conformal factor: (1 - |p|^2) / 2, enhanced with curvature.
    pub fn scale_at(&self, p: GeoPoint) -> f32 {
        enhanced_conformal_factor(p.x, p.y, self.curvature)
    }

    /// Check if a point is inside the Poincaré disk.
    pub fn is_inside(&self, p: GeoPoint) -> bool {
        (p.x * p.x + p.y * p.y) < 1.0
    }

    /// Generate points along a TRUE hyperbolic geodesic (circular arc) between two points.
    /// Unlike linear interpolation, this follows the actual shortest path on the
    /// Poincaré disk, which is a circular arc perpendicular to the boundary.
    pub fn geodesic_points(&self, a: GeoPoint, b: GeoPoint, segments: usize) -> Vec<GeoPoint> {
        geodesic_arc_points(a.x, a.y, b.x, b.y, segments)
            .into_iter()
            .map(|(x, y)| GeoPoint::new(x, y))
            .collect()
    }

    /// Compute the angle sum of a triangle with vertices a, b, c.
    /// In hyperbolic geometry, the angle sum is always LESS than 180 degrees.
    /// The deficit gives the area of the triangle.
    pub fn triangle_angle_sum(&self, a: GeoPoint, b: GeoPoint, c: GeoPoint) -> f32 {
        let ab = self.distance(a, b);
        let bc = self.distance(b, c);
        let ca = self.distance(c, a);

        let hyp_cos = |x: f32| x.cosh();
        let hyp_sin = |x: f32| x.sinh();

        let cos_a = (hyp_cos(ab) * hyp_cos(ca) - hyp_cos(bc))
            / (hyp_sin(ab) * hyp_sin(ca)).max(1e-10);
        let cos_b = (hyp_cos(ab) * hyp_cos(bc) - hyp_cos(ca))
            / (hyp_sin(ab) * hyp_sin(bc)).max(1e-10);
        let cos_c = (hyp_cos(bc) * hyp_cos(ca) - hyp_cos(ab))
            / (hyp_sin(bc) * hyp_sin(ca)).max(1e-10);

        let angle_a = cos_a.clamp(-1.0, 1.0).acos().to_degrees();
        let angle_b = cos_b.clamp(-1.0, 1.0).acos().to_degrees();
        let angle_c = cos_c.clamp(-1.0, 1.0).acos().to_degrees();

        angle_a + angle_b + angle_c
    }

    /// Compute the area of a hyperbolic triangle using the angle deficit theorem.
    /// Area = R^2 * (pi - (angle_a + angle_b + angle_c)).
    pub fn triangle_area(&self, a: GeoPoint, b: GeoPoint, c: GeoPoint) -> f32 {
        let angle_sum = self.triangle_angle_sum(a, b, c);
        (180.0 - angle_sum).to_radians() / self.curvature.abs().max(0.01)
    }

    // ===================== KLEIN MODEL CONVERSION =====================

    /// Convert a point from the Poincaré disk model to the Klein (Beltrami) disk model.
    /// In the Klein model, geodesics are STRAIGHT LINES (chords of the disk),
    /// but angles are NOT preserved (not conformal).
    /// Conversion: klein = poincare * 2 / (1 + |p|^2)
    pub fn poincare_to_klein(&self, p: GeoPoint) -> GeoPoint {
        let r2 = p.x * p.x + p.y * p.y;
        let denom = 1.0 + r2;
        if denom < 1e-10 { return GeoPoint::new(0.0, 0.0); }
        GeoPoint::new(p.x * 2.0 / denom, p.y * 2.0 / denom)
    }

    /// Convert a point from the Klein model back to the Poincaré disk model.
    /// Conversion: poincare = klein / (1 + sqrt(1 - |k|^2))
    pub fn klein_to_poincare(&self, k: GeoPoint) -> GeoPoint {
        let r2 = k.x * k.x + k.y * k.y;
        if r2 >= 1.0 { return GeoPoint::new(0.999, 0.0); }
        let denom = 1.0 + (1.0 - r2).sqrt();
        if denom < 1e-10 { return GeoPoint::new(0.0, 0.0); }
        GeoPoint::new(k.x / denom, k.y / denom)
    }

    /// Get the scale factor in the Klein model at a given Klein-model point.
    /// The Klein model has a different metric: ds^2 = (dx^2 + dy^2) / (1 - r^2) + (x*dx + y*dy)^2 / ((1-r^2)^2 * r^2)
    /// Simplified visual scale: 1 / (1 - r^2)^1.5
    pub fn klein_scale_at(&self, k: GeoPoint) -> f32 {
        let r2 = k.x * k.x + k.y * k.y;
        if r2 >= 1.0 { return 0.001; }
        let base = 1.0 - r2;
        base.powf(1.5)
    }

    /// Generate geodesic points in the Klein model (straight line segment, clipped to disk).
    pub fn klein_geodesic_points(&self, a: GeoPoint, b: GeoPoint, segments: usize) -> Vec<GeoPoint> {
        let mut pts = Vec::with_capacity(segments);
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let x = a.x + (b.x - a.x) * t;
            let y = a.y + (b.y - a.y) * t;
            // Clip to disk
            let r2 = x * x + y * y;
            if r2 < 1.0 {
                pts.push(GeoPoint::new(x, y));
            }
        }
        pts
    }

    // ===================== HOROCYCLES =====================

    /// Generate points along a horocycle (a circle tangent to the boundary of the Poincaré disk).
    /// A horocycle is the limit of a circle as its center approaches the boundary.
    /// `ideal_point`: the point on the boundary where the horocycle is "anchored" (angle in radians).
    /// `offset`: how far "inward" the horocycle extends (0.0 to ~0.95).
    /// In the Poincaré model, a horocycle through a point with ideal point at angle theta
    /// is an Euclidean circle internally tangent to the boundary at that angle.
    pub fn horocycle_points(&self, ideal_angle: f32, offset: f32, segments: usize) -> Vec<GeoPoint> {
        // The ideal point on the boundary
        let ix = ideal_angle.cos();
        let iy = ideal_angle.sin();

        // Horocycle center is at: ideal_point * offset / (1 + offset)
        // But we can compute it more simply:
        // A horocycle based at ideal point (ix, iy) is a Euclidean circle
        // with center at (ix * k, iy * k) where k = 1 - offset
        // and radius = offset, internally tangent to the unit circle.
        // Actually, a cleaner parameterization:
        // The horocycle consists of all points at a fixed hyperbolic distance
        // from an ideal point. In the Poincaré model, it's a Euclidean circle
        // tangent to the boundary at the ideal point.
        
        // Center of the horocycle's Euclidean circle:
        // It passes through a point at hyperbolic distance `d` from the ideal point.
        // We use offset as a proxy: the center is at distance (1 - r_h) from origin
        // along the ideal point direction, where r_h is the Euclidean radius.
        
        let effective_r = offset.clamp(0.05, 0.9);
        // The horocycle circle center is at distance (1 - r) from origin
        let center_dist = 1.0 - effective_r;
        let cx = ix * center_dist;
        let cy = iy * center_dist;
        
        let mut pts = Vec::with_capacity(segments);
        for i in 0..segments {
            let angle = std::f32::consts::TAU * i as f32 / segments as f32;
            let px = cx + effective_r * angle.cos();
            let py = cy + effective_r * angle.sin();
            // Only keep points inside the disk
            if px * px + py * py < 0.99 {
                pts.push(GeoPoint::new(px, py));
            }
        }
        pts
    }

    // ===================== IDEAL POINTS & BOUNDARY =====================

    /// Generate evenly-spaced ideal points on the boundary of the Poincaré disk.
    /// These represent "points at infinity" in hyperbolic geometry.
    pub fn ideal_points(&self, count: usize) -> Vec<GeoPoint> {
        (0..count).map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / count as f32;
            GeoPoint::new(angle.cos(), angle.sin())
        }).collect()
    }

    /// Generate an ultraparallel line visualization: two geodesics that do not
    /// intersect inside the disk (they have a common perpendicular).
    /// Returns points for both geodesics plus the common perpendicular.
    pub fn ultraparallel_geodesics(&self, angle1: f32, angle2: f32, radius: f32, segments: usize)
        -> (Vec<GeoPoint>, Vec<GeoPoint>, Vec<GeoPoint>) 
    {
        // Two geodesics at given angles, at a given distance from center
        let a1 = GeoPoint::new(radius * angle1.cos() * 0.9, radius * angle1.sin() * 0.9);
        let b1 = GeoPoint::new(-radius * angle1.cos() * 0.9, -radius * angle1.sin() * 0.9);
        let a2 = GeoPoint::new(radius * angle2.cos() * 0.9, radius * angle2.sin() * 0.9);
        let b2 = GeoPoint::new(-radius * angle2.cos() * 0.9, -radius * angle2.sin() * 0.9);
        
        let g1 = self.geodesic_points(a1, b1, segments);
        let g2 = self.geodesic_points(a2, b2, segments);
        
        // Common perpendicular midpoint
        let mid_angle = (angle1 + angle2) / 2.0;
        let p1 = GeoPoint::new(0.2 * mid_angle.cos(), 0.2 * mid_angle.sin());
        let p2 = GeoPoint::new(-0.2 * mid_angle.cos(), -0.2 * mid_angle.sin());
        let perp = self.geodesic_points(p1, p2, segments / 2);
        
        (g1, g2, perp)
    }

    // ===================== HYPERBOLIC TILING =====================

    /// Generate a {p, q} hyperbolic tiling pattern.
    /// {p, q} means p-sided regular polygons, q meeting at each vertex.
    /// Valid when (p-2)(q-2) > 4, e.g. {7,3}, {5,4}, {3,7}.
    /// Returns vertices of the tiling polygons that fit within a given radius of center.
    /// This generates the first few rings of the tiling.
    pub fn tiling_vertices(&self, p: usize, q: usize, max_depth: usize, center: GeoPoint) -> Vec<Vec<GeoPoint>> {
        // Check if the tiling is hyperbolic: (p-2)(q-2) > 4
        let check = (p as f32 - 2.0) * (q as f32 - 2.0);
        if check <= 4.0 {
            // Not hyperbolic, return a simple regular polygon
            let mut result = Vec::new();
            let mut poly = Vec::new();
            let vertex_radius = 0.15;
            for i in 0..p {
                let angle = std::f32::consts::TAU * i as f32 / p as f32;
                poly.push(GeoPoint::new(
                    center.x + vertex_radius * angle.cos(),
                    center.y + vertex_radius * angle.sin(),
                ));
            }
            result.push(poly);
            return result;
        }

        // Compute the edge length of the regular p-gon in hyperbolic geometry
        // such that q of them meet at each vertex.
        // The interior angle of the polygon is 2*pi/q.
        // Using the hyperbolic law of cosines for a right triangle from center to edge midpoint to vertex.
        
        // Angle at vertex: pi/q (half of interior angle)
        let half_vertex_angle = std::f32::consts::PI / q as f32;
        // The central angle of each edge: pi/p
        let central_angle = std::f32::consts::PI / p as f32;
        
        // Edge length via hyperbolic trig:
        // cosh(edge/2) = cos(central_angle) / sin(half_vertex_angle) 
        let cosh_half_edge = central_angle.cos() / half_vertex_angle.sin();
        let half_edge = if cosh_half_edge > 1.0 { cosh_half_edge.ln().sqrt() } else { 0.3 };
        let edge_length = half_edge * 2.0;
        
        // Apothem (distance from center to edge midpoint)
        // cosh(apothem) = cos(central_angle) * cosh(half_edge) ... not quite right
        // Simpler: use the Poincaré model where the center of the tile is at origin
        // The vertex radius in the Poincaré disk model:
        // tanh(vertex_dist/2) gives the Euclidean distance from center to vertex
        
        // For the first polygon, place vertices at equal angular spacing
        // We need to figure out the Euclidean radius of the polygon in the Poincaré disk
        // Using: if hyperbolic distance from center to vertex is R,
        // then Euclidean distance = tanh(R/2)
        // And R relates to edge length via: cosh(R) = cos(pi/p) / sin(pi/q)
        let cosh_R = (std::f32::consts::PI / p as f32).cos() / (std::f32::consts::PI / q as f32).sin();
        let R = if cosh_R > 1.0 { cosh_R.ln().sqrt() } else { 0.3 };
        let euclidean_r = (R * 0.5).tanh();
        
        let mut all_polygons = Vec::new();
        
        // Generate the central polygon
        let mut central_poly = Vec::new();
        for i in 0..p {
            let angle = std::f32::consts::TAU * i as f32 / p as f32;
            central_poly.push(GeoPoint::new(
                center.x + euclidean_r * angle.cos(),
                center.y + euclidean_r * angle.sin(),
            ));
        }
        all_polygons.push(central_poly);
        
        // Generate adjacent polygons by reflecting across edges (simplified: rotate & translate)
        if max_depth >= 1 {
            for edge_i in 0..p {
                let angle1 = std::f32::consts::TAU * edge_i as f32 / p as f32;
                let angle2 = std::f32::consts::TAU * (edge_i + 1) as f32 / p as f32;
                
                // Edge midpoint
                let mx = center.x + euclidean_r * 0.5 * (angle1.cos() + angle2.cos());
                let my = center.y + euclidean_r * 0.5 * (angle1.sin() + angle2.sin());
                
                // Center of adjacent polygon: reflect center across the edge
                let dx = mx - center.x;
                let dy = my - center.y;
                let adj_cx = mx + dx;
                let adj_cy = my + dy;
                
                // Check if the adjacent center is inside the disk
                if adj_cx * adj_cx + adj_cy * adj_cy < 0.92 {
                    // The interior angle at each vertex is 2*pi/q
                    // The rotation for the adjacent polygon
                    let edge_angle = (angle1 + angle2) / 2.0;
                    let rotation = edge_angle + std::f32::consts::PI;
                    
                    let mut adj_poly = Vec::new();
                    for i in 0..p {
                        let a = std::f32::consts::TAU * i as f32 / p as f32 + rotation;
                        adj_poly.push(GeoPoint::new(
                            adj_cx + euclidean_r * a.cos(),
                            adj_cy + euclidean_r * a.sin(),
                        ));
                    }
                    all_polygons.push(adj_poly);
                }
            }
        }
        
        // Third ring
        if max_depth >= 2 {
            let second_start = all_polygons.len();
            for poly_idx in 1..second_start {
                let poly = &all_polygons[poly_idx];
                if poly.is_empty() { continue; }
                let pcx: f32 = poly.iter().map(|p| p.x).sum::<f32>() / poly.len() as f32;
                let pcy: f32 = poly.iter().map(|p| p.y).sum::<f32>() / poly.len() as f32;
                
                for edge_i in 0..p {
                    let angle1 = std::f32::consts::TAU * edge_i as f32 / p as f32 +
                        (poly[0].x - pcx).atan2(poly[0].y - pcy);
                    let angle2 = std::f32::consts::TAU * (edge_i + 1) as f32 / p as f32 +
                        (poly[0].x - pcx).atan2(poly[0].y - pcy);
                    
                    let v1 = &poly[edge_i];
                    let v2 = &poly[(edge_i + 1) % p];
                    let emx = (v1.x + v2.x) * 0.5;
                    let emy = (v1.y + v2.y) * 0.5;
                    
                    let ddx = emx - pcx;
                    let ddy = emy - pcy;
                    let acx = emx + ddx;
                    let acy = emy + ddy;
                    
                    if acx * acx + acy * acy < 0.88 {
                        let ea = (angle1 + angle2) / 2.0 + std::f32::consts::PI;
                        let mut ap = Vec::new();
                        for i in 0..p {
                            let a = std::f32::consts::TAU * i as f32 / p as f32 + ea;
                            ap.push(GeoPoint::new(acx + euclidean_r * a.cos(), acy + euclidean_r * a.sin()));
                        }
                        all_polygons.push(ap);
                    }
                }
            }
        }
        
        all_polygons
    }

    // ===================== CURVATURE VISUALIZATION =====================

    /// Compute the Gaussian curvature at a point. For the Poincaré disk
    /// with standard curvature parameter, this is constant: -1/R^2.
    pub fn gaussian_curvature(&self) -> f32 {
        -1.0 / self.curvature.abs().max(0.01)
    }

    /// Compute the circumference of a hyperbolic circle of given radius.
    /// In hyperbolic geometry, circumference = 2*pi*sinh(r) (for curvature = -1).
    /// This grows EXPONENTIALLY, unlike Euclidean 2*pi*r.
    pub fn circle_circumference(&self, hyperbolic_radius: f32) -> f32 {
        2.0 * std::f32::consts::PI * hyperbolic_radius.sinh() / self.curvature.abs().sqrt().max(0.01)
    }

    /// Compute the area of a hyperbolic circle of given radius.
    /// Area = 2*pi*(cosh(r) - 1) = 4*pi*sinh^2(r/2) (for curvature = -1).
    pub fn circle_area(&self, hyperbolic_radius: f32) -> f32 {
        let r_scaled = hyperbolic_radius / self.curvature.abs().sqrt().max(0.01);
        2.0 * std::f32::consts::PI * (r_scaled.cosh() - 1.0)
    }

    /// Generate the boundary circle points for the Poincaré disk.
    /// Useful for drawing the "edge of infinity."
    pub fn boundary_points(&self, segments: usize) -> Vec<GeoPoint> {
        (0..=segments).map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / segments as f32;
            GeoPoint::new(angle.cos() * 0.999, angle.sin() * 0.999)
        }).collect()
    }

    /// Compute the hyperbolic midpoint between two points.
    /// This is NOT the Euclidean midpoint — it's the point equidistant
    /// from both a and b along the geodesic connecting them.
    pub fn midpoint(&self, a: GeoPoint, b: GeoPoint) -> GeoPoint {
        // In the Poincaré model, the hyperbolic midpoint along a geodesic
        // is found by computing the geodesic arc and taking the middle.
        let pts = self.geodesic_points(a, b, 20);
        if pts.len() >= 2 {
            let mid = pts.len() / 2;
            pts[mid]
        } else {
            GeoPoint::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
        }
    }

    /// Reflect a point across a geodesic defined by two points.
    /// In hyperbolic geometry, reflection is an isometry (distance-preserving).
    pub fn reflect_across_geodesic(&self, p: GeoPoint, g1: GeoPoint, g2: GeoPoint) -> GeoPoint {
        // Compute the geodesic as a circular arc
        let dx = g2.x - g1.x;
        let dy = g2.y - g1.y;
        let cross = g1.y * dx - g1.x * dy;
        
        if cross.abs() < 1e-6 {
            // Geodesic passes through origin — it's a diameter
            // Reflect across the line through g1 and g2
            let angle = (g2.y - g1.y).atan2(g2.x - g1.x);
            let cos2a = (2.0 * angle).cos();
            let sin2a = (2.0 * angle).sin();
            let rx = p.x * cos2a + p.y * sin2a;
            let ry = p.x * sin2a - p.y * cos2a;
            return GeoPoint::new(rx, ry);
        }
        
        // General case: geodesic is a circular arc
        // The inversion circle has center (cx, cy) and radius r
        let mx = (g1.x + g2.x) / 2.0;
        let my = (g1.y + g2.y) / 2.0;
        let t = (1.0 - g1.x * g2.x - g1.y * g2.y) / (2.0 * cross);
        let cx = mx - t * dy;
        let cy = my + t * dx;
        let r_sq = cx * cx + cy * cy - 1.0;
        
        if r_sq <= 0.001 {
            // Fallback: Euclidean-like reflection
            let dx = g2.x - g1.x;
            let dy = g2.y - g1.y;
            let len = (dx * dx + dy * dy).sqrt();
            if len < 1e-10 { return p; }
            let nx = -dy / len;
            let ny = dx / len;
            let d = (p.x - g1.x) * nx + (p.y - g1.y) * ny;
            return GeoPoint::new(p.x - 2.0 * d * nx, p.y - 2.0 * d * ny);
        }
        
        // Reflect point p by inverting through the circle (cx, cy, r)
        // then inverting again through the unit circle
        let dpx = p.x - cx;
        let dpy = p.y - cy;
        let dp_sq = dpx * dpx + dpy * dpy;
        let r = r_sq.sqrt();
        if dp_sq < 1e-10 { return p; }
        
        // Inversion through the geodesic's circle
        let inv_x = cx + r_sq * dpx / dp_sq;
        let inv_y = cy + r_sq * dpy / dp_sq;
        
        // Inversion through the unit circle (to get back to Poincaré disk)
        let inv_sq = inv_x * inv_x + inv_y * inv_y;
        if inv_sq < 1e-10 { return GeoPoint::new(0.0, 0.0); }
        
        GeoPoint::new(inv_x / inv_sq, inv_y / inv_sq)
    }
}