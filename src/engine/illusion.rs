use macroquad::prelude::*;

/// Computed geodesic arc parameters in the Poincaré disk model.
pub struct GeodesicArc {
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub start_angle: f32,
    pub end_angle: f32,
}

/// Compute the circular arc representing a hyperbolic geodesic between two
/// points in the Poincaré disk model.
pub fn compute_geodesic_arc(ax: f32, ay: f32, bx: f32, by: f32) -> Option<GeodesicArc> {
    let dx = bx - ax;
    let dy = by - ay;
    let cross = ay * dx - ax * dy;

    if cross.abs() < 1e-6 {
        return None;
    }

    let mx = (ax + bx) / 2.0;
    let my = (ay + by) / 2.0;
    let t = (1.0 - ax * bx - ay * by) / (2.0 * cross);

    let cx = mx - t * dy;
    let cy = my + t * dx;
    let r_sq = cx * cx + cy * cy - 1.0;
    if r_sq <= 0.001 { return None; }
    let r = r_sq.sqrt();

    let start_angle = (ay - cy).atan2(ax - cx);
    let end_angle = (by - cy).atan2(bx - cx);

    let mid1 = (start_angle + end_angle) / 2.0;
    let mid2 = mid1 + std::f32::consts::PI;
    let d1 = (cx + r * mid1.cos()).powi(2) + (cy + r * mid1.sin()).powi(2);
    let d2 = (cx + r * mid2.cos()).powi(2) + (cy + r * mid2.sin()).powi(2);

    let (sa, ea) = if d1 < d2 { (start_angle, end_angle) } else { (end_angle, start_angle) };
    Some(GeodesicArc { cx, cy, r, start_angle: sa, end_angle: ea })
}

/// Draw a hyperbolic geodesic arc between two world-space points.
pub fn draw_geodesic_line(
    ax: f32, ay: f32, bx: f32, by: f32,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, thickness: f32,
    segments: usize,
) {
    if let Some(arc) = compute_geodesic_arc(ax, ay, bx, by) {
        let mut prev_x = ax;
        let mut prev_y = ay;
        for i in 1..=segments {
            let t = i as f32 / segments as f32;
            let angle = arc.start_angle + (arc.end_angle - arc.start_angle) * t;
            let px = arc.cx + arc.r * angle.cos();
            let py = arc.cy + arc.r * angle.sin();
            crate::engine::renderer::draw_line_world(
                prev_x, prev_y, px, py,
                cam_x, cam_y, zoom, sw, sh, color, thickness,
            );
            prev_x = px;
            prev_y = py;
        }
    } else {
        crate::engine::renderer::draw_line_world(
            ax, ay, bx, by, cam_x, cam_y, zoom, sw, sh, color, thickness,
        );
    }
}

/// Draw a geodesic line with conformal scaling (thinner near edge, thicker near center).
pub fn draw_geodesic_line_conformal(
    ax: f32, ay: f32, bx: f32, by: f32,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, base_thickness: f32,
    segments: usize,
) {
    if let Some(arc) = compute_geodesic_arc(ax, ay, bx, by) {
        let mut prev_x = ax;
        let mut prev_y = ay;
        for i in 1..=segments {
            let t = i as f32 / segments as f32;
            let angle = arc.start_angle + (arc.end_angle - arc.start_angle) * t;
            let px = arc.cx + arc.r * angle.cos();
            let py = arc.cy + arc.r * angle.sin();

            // Conformal factor at midpoint of this segment
            let mx = (prev_x + px) / 2.0;
            let my = (prev_y + py) / 2.0;
            let r2 = mx * mx + my * my;
            let conformal = if r2 < 1.0 { (1.0 - r2) / 2.0 } else { 0.01 };
            let thickness = base_thickness * (conformal * 3.0).max(0.15);
            let alpha = (conformal * 2.0).max(0.1).min(1.0) * color.a;

            crate::engine::renderer::draw_line_world(
                prev_x, prev_y, px, py,
                cam_x, cam_y, zoom, sw, sh,
                Color::new(color.r, color.g, color.b, alpha),
                thickness,
            );
            prev_x = px;
            prev_y = py;
        }
    } else {
        crate::engine::renderer::draw_line_world(
            ax, ay, bx, by, cam_x, cam_y, zoom, sw, sh, color, base_thickness,
        );
    }
}

/// Draw a concentric ring in hyperbolic space using geodesic arcs.
pub fn draw_geodesic_ring(
    center_x: f32, center_y: f32, radius: f32, sides: usize,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, thickness: f32,
    segments_per_arc: usize,
) {
    for i in 0..sides {
        let a1 = std::f32::consts::TAU * i as f32 / sides as f32;
        let a2 = std::f32::consts::TAU * (i + 1) as f32 / sides as f32;
        let x1 = center_x + radius * a1.cos();
        let y1 = center_y + radius * a1.sin();
        let x2 = center_x + radius * a2.cos();
        let y2 = center_y + radius * a2.sin();
        draw_geodesic_line(x1, y1, x2, y2, cam_x, cam_y, zoom, sw, sh, color, thickness, segments_per_arc);
    }
}

/// Draw a geodesic ring with conformal scaling.
pub fn draw_geodesic_ring_conformal(
    center_x: f32, center_y: f32, radius: f32, sides: usize,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, thickness: f32,
    segments_per_arc: usize,
) {
    for i in 0..sides {
        let a1 = std::f32::consts::TAU * i as f32 / sides as f32;
        let a2 = std::f32::consts::TAU * (i + 1) as f32 / sides as f32;
        let x1 = center_x + radius * a1.cos();
        let y1 = center_y + radius * a1.sin();
        let x2 = center_x + radius * a2.cos();
        let y2 = center_y + radius * a2.sin();
        draw_geodesic_line_conformal(x1, y1, x2, y2, cam_x, cam_y, zoom, sw, sh, color, thickness, segments_per_arc);
    }
}

/// Draw a radial geodesic line.
pub fn draw_geodesic_radial(
    cx: f32, cy: f32, angle: f32, inner_r: f32, outer_r: f32,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, thickness: f32,
) {
    let x1 = cx + inner_r * angle.cos();
    let y1 = cy + inner_r * angle.sin();
    let x2 = cx + outer_r * angle.cos();
    let y2 = cy + outer_r * angle.sin();
    draw_geodesic_line(x1, y1, x2, y2, cam_x, cam_y, zoom, sw, sh, color, thickness, 12);
}

/// Draw a radial geodesic with conformal scaling.
pub fn draw_geodesic_radial_conformal(
    cx: f32, cy: f32, angle: f32, inner_r: f32, outer_r: f32,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, thickness: f32,
) {
    let x1 = cx + inner_r * angle.cos();
    let y1 = cy + inner_r * angle.sin();
    let x2 = cx + outer_r * angle.cos();
    let y2 = cy + outer_r * angle.sin();
    draw_geodesic_line_conformal(x1, y1, x2, y2, cam_x, cam_y, zoom, sw, sh, color, thickness, 12);
}

// ==================== PORTAL ILLUSION SYSTEM ====================

/// State for animated portal illusions.
pub struct PortalIllusion {
    pub warp_time: f32,
    pub spiral_angle: f32,
    pub ring_count: usize,
}

impl PortalIllusion {
    pub fn new() -> Self {
        Self { warp_time: 0.0, spiral_angle: 0.0, ring_count: 8 }
    }

    pub fn update(&mut self, dt: f32) {
        self.warp_time += dt;
        self.spiral_angle += dt * 2.5;
    }
}

/// Draw a portal with spatial warp illusion.
pub fn draw_portal_warp(
    px: f32, py: f32, radius: f32,
    illusion: &PortalIllusion,
    color: Color,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
) {
    // Outer glow
    let glow_pulse = (illusion.warp_time * 1.5).sin() * 0.02 + radius * 2.5;
    crate::engine::renderer::draw_circle_world(px, py, glow_pulse, cam_x, cam_y, zoom, sw, sh,
        Color::new(color.r, color.g, color.b, 0.08));

    // Spatial compression rings
    for i in 0..illusion.ring_count {
        let t = i as f32 / illusion.ring_count as f32;
        let ring_r = radius * (1.0 - t * t);
        if ring_r < 0.002 { continue; }
        let alpha = 0.15 + 0.3 * (1.0 - t);
        let wobble = (illusion.warp_time * 3.0 + i as f32 * 0.7).sin() * 0.003;
        crate::engine::renderer::draw_circle_outline_world(
            px, py, ring_r + wobble,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r, color.g, color.b, alpha),
            1.5 - t,
        );
    }

    // Spiral arms
    let spiral_segments = 40;
    for arm in 0..3 {
        let arm_offset = arm as f32 * std::f32::consts::TAU / 3.0;
        let mut prev_sx = 0.0;
        let mut prev_sy = 0.0;
        for i in 1..=spiral_segments {
            let t = i as f32 / spiral_segments as f32;
            let angle = illusion.spiral_angle + arm_offset + t * std::f32::consts::TAU * 1.5;
            let r = t * radius * 0.85;
            let wx = px + r * angle.cos();
            let wy = py + r * angle.sin();
            let alpha = (1.0 - t) * 0.5;
            let half_w = sw * 0.45 * zoom;
            let half_h = sh * 0.45 * zoom;
            let sx = sw / 2.0 + (wx - cam_x) * half_w;
            let sy = sh / 2.0 + (wy - cam_y) * half_h;
            if i > 1 {
                let sr = (0.5 * zoom * half_w.min(half_h)).max(1.0);
                draw_line(prev_sx, prev_sy, sx, sy, sr * 0.015,
                    Color::new(color.r, color.g, color.b, alpha));
            }
            prev_sx = sx;
            prev_sy = sy;
        }
    }

    // Event horizon
    let center_pulse = (illusion.warp_time * 2.0).sin() * 0.005;
    crate::engine::renderer::draw_circle_world(px, py, radius * 0.18 + center_pulse,
        cam_x, cam_y, zoom, sw, sh, Color::new(0.0, 0.0, 0.05, 0.9));
}

/// Draw a "window" into the other side of a portal.
pub fn draw_portal_window(
    portal_x: f32, portal_y: f32, portal_radius: f32,
    target_x: f32, target_y: f32,
    collectibles_near_target: &[(f32, f32, bool, char)],
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    time: f32,
) {
    crate::engine::renderer::draw_circle_world(
        portal_x, portal_y, portal_radius * 0.95,
        cam_x, cam_y, zoom, sw, sh, Color::new(0.02, 0.0, 0.08, 0.95));

    let grid_alpha = 0.15 + (time * 0.5).sin() * 0.05;
    for i in -3..=3 {
        let offset = i as f32 * 0.06;
        let rel_y = offset / 0.25 * portal_radius * 0.7;
        let yfrac = rel_y / (portal_radius * 0.7);
        let warp = 1.0 - yfrac * yfrac * 0.3;
        crate::engine::renderer::draw_line_world(
            portal_x - portal_radius * 0.7 * warp, portal_y + rel_y,
            portal_x + portal_radius * 0.7 * warp, portal_y + rel_y,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(0.3, 0.1, 0.5, grid_alpha), 0.5);
        let rel_x = offset / 0.25 * portal_radius * 0.7;
        let xfrac = rel_x / (portal_radius * 0.7);
        let warp_v = 1.0 - xfrac * xfrac * 0.3;
        crate::engine::renderer::draw_line_world(
            portal_x + rel_x, portal_y - portal_radius * 0.7 * warp_v,
            portal_x + rel_x, portal_y + portal_radius * 0.7 * warp_v,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(0.3, 0.1, 0.5, grid_alpha), 0.5);
    }

    let view_range = 0.4;
    for &(cx, cy, collected, label) in collectibles_near_target {
        if collected { continue; }
        let rel_x = (cx - target_x) / view_range * portal_radius * 0.7;
        let rel_y = (cy - target_y) / view_range * portal_radius * 0.7;
        let dist_from_center = (rel_x * rel_x + rel_y * rel_y).sqrt();
        if dist_from_center < portal_radius * 0.75 {
            crate::engine::renderer::draw_circle_world(
                portal_x + rel_x, portal_y + rel_y, 0.01,
                cam_x, cam_y, zoom, sw, sh,
                Color::new(1.0, 0.8, 0.0, 0.7));
            crate::engine::renderer::draw_text_world(
                &label.to_string(),
                portal_x + rel_x - 0.012, portal_y + rel_y - 0.02,
                cam_x, cam_y, zoom, sw, sh, 0.025,
                Color::new(1.0, 0.8, 0.0, 0.5));
        }
    }

    let label_alpha = 0.3 + (time * 2.0).sin() * 0.15;
    crate::engine::renderer::draw_text_world(
        "...", portal_x - 0.01, portal_y + portal_radius * 0.55,
        cam_x, cam_y, zoom, sw, sh, 0.02,
        Color::new(0.5, 0.3, 0.8, label_alpha));
}

// ==================== RECURSIVE PORTAL EFFECT ====================

/// Draw a recursive portal-in-portal effect.
/// The portal appears to contain a smaller version of itself, infinitely.
pub fn draw_recursive_portal(
    px: f32, py: f32, radius: f32,
    target_px: f32, target_py: f32,
    color: Color, time: f32, depth: usize,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
) {
    if depth == 0 { return; }
    if radius < 0.005 { return; }

    // Draw the outer portal ring
    let pulse = (time * 2.5 + depth as f32).sin() * 0.003;
    crate::engine::renderer::draw_circle_outline_world(
        px, py, radius + pulse,
        cam_x, cam_y, zoom, sw, sh,
        Color::new(color.r, color.g, color.b, 0.6 / depth as f32),
        2.0 / depth as f32,
    );

    // Dark fill for the "interior"
    crate::engine::renderer::draw_circle_world(
        px, py, radius * 0.92,
        cam_x, cam_y, zoom, sw, sh,
        Color::new(0.01, 0.0, 0.03, 0.9 / depth as f32),
    );

    // Inside this portal, draw a smaller version of the TARGET portal
    // This creates the "infinite regression" illusion
    let shrink = 0.55 + 0.05 * (time * 0.8).sin();
    let inner_radius = radius * shrink;

    // Offset toward the center of the portal to suggest depth
    let depth_offset_x = (target_px - px) * 0.2;
    let depth_offset_y = (target_py - py) * 0.2;

    // Rotating spiral inside
    let spiral_count = 4;
    for arm in 0..spiral_count {
        let arm_angle = std::f32::consts::TAU * arm as f32 / spiral_count as f32 + time * (1.5 + depth as f32 * 0.3);
        for i in 1..=8 {
            let t = i as f32 / 8.0;
            let r = t * radius * 0.4;
            let a = arm_angle + t * std::f32::consts::PI * 2.0;
            let sx = px + r * a.cos();
            let sy = py + r * a.sin();
            let alpha = (1.0 - t) * 0.3 / depth as f32;
            crate::engine::renderer::draw_circle_world(
                sx, sy, 0.002, cam_x, cam_y, zoom, sw, sh,
                Color::new(color.r, color.g, color.b, alpha),
            );
        }
    }

    // Recurse into the "other side"
    draw_recursive_portal(
        px + depth_offset_x, py + depth_offset_y,
        inner_radius,
        px, py,  // "target" becomes this portal (mirror effect)
        color, time, depth - 1,
        cam_x, cam_y, zoom, sw, sh,
    );
}

// ==================== CURVATURE FIELD VISUALIZATION ====================

/// Draw a field of small arrows showing how space curves.
/// In hyperbolic space, arrows diverge (negative curvature).
/// In spherical space, arrows converge (positive curvature).
pub fn draw_curvature_field(
    center_x: f32, center_y: f32,
    field_radius: f32,
    curvature: f32,  // negative = hyperbolic, positive = spherical, 0 = flat
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    let grid_count = 7;
    for iy in 0..grid_count {
        for ix in 0..grid_count {
            let fx = -1.0 + 2.0 * ix as f32 / (grid_count - 1) as f32;
            let fy = -1.0 + 2.0 * iy as f32 / (grid_count - 1) as f32;

            let wx = center_x + fx * field_radius;
            let wy = center_y + fy * field_radius;

            // Distance from center determines curvature effect
            let dist = (fx * fx + fy * fy).sqrt();
            if dist < 0.1 { continue; }

            // Direction: radially outward from center, then curved by curvature
            let base_angle = (fy).atan2(fx);

            // Curvature deflects the arrow: negative = diverge, positive = converge
            let deflect = curvature * dist * 0.5;
            let angle = base_angle + deflect;

            // Arrow length varies with distance (longer near edge for hyperbolic, shorter for spherical)
            let len = field_radius * 0.08 * (1.0 + curvature * dist * 0.3);

            let ax = wx;
            let ay = wy;
            let bx = wx + len * angle.cos();
            let by = wy + len * angle.sin();

            let alpha = 0.2 + 0.15 * (1.0 - dist);
            let wave = (time * 2.0 + dist * 3.0).sin() * 0.05;
            let arrow_color = Color::new(
                color.r,
                color.g + wave,
                color.b,
                (alpha + wave).max(0.05),
            );

            crate::engine::renderer::draw_line_world(ax, ay, bx, by, cam_x, cam_y, zoom, sw, sh, arrow_color, 1.0);

            // Arrowhead
            let head_len = len * 0.35;
            let ha1 = angle + 2.6;
            let ha2 = angle - 2.6;
            crate::engine::renderer::draw_line_world(bx, by, bx + head_len * ha1.cos(), by + head_len * ha1.sin(),
                cam_x, cam_y, zoom, sw, sh, arrow_color, 0.8);
            crate::engine::renderer::draw_line_world(bx, by, bx + head_len * ha2.cos(), by + head_len * ha2.sin(),
                cam_x, cam_y, zoom, sw, sh, arrow_color, 0.8);
        }
    }
}

// ==================== PARALLEL LINE DEVIATION ====================

/// Draw parallel lines that diverge (hyperbolic) or converge (spherical).
/// This is THE defining visual difference between the three geometries:
/// - Euclidean: parallel lines stay parallel forever
/// - Hyperbolic: parallel lines diverge
/// - Spherical: parallel lines converge and meet
pub fn draw_parallel_deviation(
    start_x: f32, start_y: f32,
    direction_angle: f32,
    spacing: f32,
    line_length: f32,
    curvature: f32,  // -1 = hyperbolic, 0 = euclidean, 1 = spherical
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    let line_count = 5;
    let segments = 20;

    for line_i in 0..line_count {
        let offset = (line_i as f32 - (line_count - 1) as f32 / 2.0) * spacing;
        let perp_angle = direction_angle + std::f32::consts::FRAC_PI_2;

        let mut prev_x = start_x + offset * perp_angle.cos();
        let mut prev_y = start_y + offset * perp_angle.sin();

        for seg in 1..=segments {
            let t = seg as f32 / segments as f32;
            let dist_along = t * line_length;

            // In flat space, lines stay parallel. With curvature, they deviate.
            // Hyperbolic (curvature < 0): lines diverge → spacing increases with distance
            // Spherical (curvature > 0): lines converge → spacing decreases with distance
            let deviation = curvature * offset * t * t * spacing * 2.0;

            let x = start_x + offset * perp_angle.cos() + deviation * perp_angle.cos()
                    + dist_along * direction_angle.cos();
            let y = start_y + offset * perp_angle.sin() + deviation * perp_angle.sin()
                    + dist_along * direction_angle.sin();

            let alpha = 0.4 + 0.2 * (1.0 - t) + (time * 1.5 + line_i as f32 * 0.3).sin() * 0.1;
            crate::engine::renderer::draw_line_world(
                prev_x, prev_y, x, y,
                cam_x, cam_y, zoom, sw, sh,
                Color::new(color.r, color.g, color.b, alpha),
                1.5,
            );
            prev_x = x;
            prev_y = y;
        }
    }

    // Label showing what's happening
    let label = match curvature {
        c if c < -0.1 => "Lines DIVERGE (Hyperbolic)",
        c if c > 0.1 => "Lines CONVERGE (Spherical)",
        _ => "Lines stay PARALLEL (Euclidean)",
    };
    crate::engine::renderer::draw_text_world(
        label, start_x - 0.05, start_y - spacing * (line_count as f32 / 2.0) - 0.04,
        cam_x, cam_y, zoom, sw, sh, 0.025,
        Color::new(color.r, color.g, color.b, 0.6 + time.sin() * 0.15),
    );
}

// ==================== WORMHOLE CORRIDOR ====================

/// Draw a visual "corridor" between two portal endpoints.
/// Creates the illusion of a tunnel through curved spacetime,
/// with walls that compress and expand along the path.
pub fn draw_wormhole_corridor(
    ax: f32, ay: f32, bx: f32, by: f32,
    corridor_width: f32,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    let segments = 24;
    let dx = bx - ax;
    let dy = by - ay;
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let dir_x = dx / len;
    let dir_y = dy / len;
    let perp_x = -dir_y;
    let perp_y = dir_x;

    // Left and right walls of the corridor
    for wall_side in 0..2 {
        let sign = if wall_side == 0 { 1.0 } else { -1.0 };
        let mut prev_x = 0.0;
        let mut prev_y = 0.0;

        for i in 1..=segments {
            let t = i as f32 / segments as f32;

            // Corridor narrows in the middle (wormhole throat)
            let throat = 1.0 - 0.6 * (1.0 - (2.0 * t - 1.0).powi(2));
            let width = corridor_width * throat;

            // Sinusoidal wave along the corridor
            let wave = (time * 3.0 + t * 8.0).sin() * 0.003;

            let cx = ax + dx * t + sign * (width + wave) * perp_x;
            let cy = ay + dy * t + sign * (width + wave) * perp_y;

            if i > 1 {
                let alpha = 0.15 + 0.1 * throat;
                crate::engine::renderer::draw_line_world(
                    prev_x, prev_y, cx, cy,
                    cam_x, cam_y, zoom, sw, sh,
                    Color::new(color.r, color.g, color.b, alpha),
                    1.5,
                );
            }
            prev_x = cx;
            prev_y = cy;
        }
    }

    // Cross-sections (ribs) along the corridor
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let throat = 1.0 - 0.6 * (1.0 - (2.0 * t - 1.0).powi(2));
        let width = corridor_width * throat;

        let mx = ax + dx * t;
        let my = ay + dy * t;
        let x1 = mx + width * perp_x;
        let y1 = my + width * perp_y;
        let x2 = mx - width * perp_x;
        let y2 = my - width * perp_y;

        let alpha = 0.05 + 0.05 * (time * 2.0 + t * 5.0).sin();
        crate::engine::renderer::draw_line_world(
            x1, y1, x2, y2,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r, color.g, color.b, alpha),
            0.8,
        );
    }

    // Traveling particles through the corridor
    for p in 0..3 {
        let pt = (time * 0.4 + p as f32 * 0.33) % 1.0;
        let throat = 1.0 - 0.6 * (1.0 - (2.0 * pt - 1.0).powi(2));
        let lateral = (time * 5.0 + p as f32 * 2.0).sin() * corridor_width * throat * 0.5;

        let ppx = ax + dx * pt + lateral * perp_x;
        let ppy = ay + dy * pt + lateral * perp_y;

        crate::engine::renderer::draw_circle_world(
            ppx, ppy, 0.008,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r, color.g, color.b, 0.6),
        );
    }
}

// ==================== IMPOSSIBLE GEOMETRY DECORATIONS ====================

/// Draw a Penrose (impossible) triangle.
pub fn draw_impossible_triangle(
    x: f32, y: f32, size: f32,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    let rotation = time * 0.3;
    let mut verts = Vec::new();
    for i in 0..3 {
        let angle = rotation + std::f32::consts::TAU * i as f32 / 3.0 - std::f32::consts::FRAC_PI_2;
        verts.push((x + size * angle.cos(), y + size * angle.sin()));
    }
    let offset = size * 0.1;
    let mut normals = Vec::new();
    for i in 0..3 {
        let j = (i + 1) % 3;
        let dx = verts[j].0 - verts[i].0;
        let dy = verts[j].1 - verts[i].1;
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        normals.push((-dy / len, dx / len));
    }
    for i in 0..3 {
        let j = (i + 1) % 3;
        let nx = normals[i].0 * offset;
        let ny = normals[i].1 * offset;
        crate::engine::renderer::draw_line_world(
            verts[i].0 + nx, verts[i].1 + ny, verts[j].0 + nx, verts[j].1 + ny,
            cam_x, cam_y, zoom, sw, sh, color, 2.0);
        crate::engine::renderer::draw_line_world(
            verts[i].0 - nx, verts[i].1 - ny, verts[j].0 - nx, verts[j].1 - ny,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r * 0.6, color.g * 0.6, color.b * 0.6, color.a * 0.7), 1.5);
        crate::engine::renderer::draw_line_world(
            verts[j].0 + nx, verts[j].1 + ny, verts[j].0 - nx, verts[j].1 - ny,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r * 0.8, color.g * 0.8, color.b * 0.8, color.a * 0.5), 1.5);
    }
    for i in 0..3 {
        let prev = (i + 2) % 3;
        let nx_prev = normals[prev].0 * offset;
        let ny_prev = normals[prev].1 * offset;
        let nx_curr = normals[i].0 * offset;
        let ny_curr = normals[i].1 * offset;
        crate::engine::renderer::draw_line_world(
            verts[i].0 - nx_prev, verts[i].1 - ny_prev,
            verts[i].0 + nx_curr, verts[i].1 + ny_curr,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r, color.g, color.b, 0.25), 1.0);
    }
    let pulse = (time * 2.0).sin() * 0.3 + 0.7;
    crate::engine::renderer::draw_circle_world(
        x, y, size * 0.08, cam_x, cam_y, zoom, sw, sh,
        Color::new(color.r, color.g, color.b, 0.25 * pulse));
}

/// Draw a rotating tesseract (4D hypercube) projected into 2D.
pub fn draw_tesseract(
    x: f32, y: f32, size: f32,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    let angle4d = time * 0.5;
    let angle3d = time * 0.3;
    let cos4 = angle4d.cos();
    let sin4 = angle4d.sin();
    let cos3 = angle3d.cos();
    let sin3 = angle3d.sin();

    let mut projected = Vec::new();
    for i in 0..16u32 {
        let x4 = if (i & 1) != 0 { 1.0 } else { -1.0 };
        let y4 = if (i & 2) != 0 { 1.0 } else { -1.0 };
        let z4 = if (i & 4) != 0 { 1.0 } else { -1.0 };
        let w4 = if (i & 8) != 0 { 1.0 } else { -1.0 };
        let rx = x4 * cos4 - w4 * sin4;
        let rw = x4 * sin4 + w4 * cos4;
        let ry = y4 * cos3 - z4 * sin3;
        let rz = y4 * sin3 + z4 * cos3;
        let pf1 = 2.0 / (2.0 - rw * 0.3);
        let px3 = rx * pf1;
        let py3 = ry * pf1;
        let pz3 = rz * pf1;
        let pf2 = 3.0 / (3.0 - pz3 * 0.3);
        let px2 = px3 * pf2;
        let py2 = py3 * pf2;
        projected.push((x + px2 * size * 0.5, y + py2 * size * 0.5));
    }

    let edges: [(usize, usize); 32] = [
        (0,1),(0,2),(0,4),(1,3),(1,5),(2,3),(2,6),(3,7),(4,5),(4,6),(5,7),(6,7),
        (8,9),(8,10),(8,12),(9,11),(9,13),(10,11),(10,14),(11,15),(12,13),(12,14),(13,15),(14,15),
        (0,8),(1,9),(2,10),(3,11),(4,12),(5,13),(6,14),(7,15),
    ];

    for &(a, b) in &edges {
        let is_connect = a < 8 && b >= 8;
        let alpha = if is_connect { 0.15 + (time * 1.5).sin() * 0.08 } else { 0.45 };
        let thickness = if is_connect { 1.0 } else { 1.5 };
        crate::engine::renderer::draw_line_world(
            projected[a].0, projected[a].1, projected[b].0, projected[b].1,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r, color.g, color.b, alpha), thickness,
        );
    }

    let pulse = (time * 1.8).sin() * 0.15 + 0.25;
    crate::engine::renderer::draw_circle_world(
        x, y, size * 0.12, cam_x, cam_y, zoom, sw, sh,
        Color::new(color.r, color.g, color.b, pulse));
}

/// Draw an impossible staircase (Penrose stairs).
pub fn draw_impossible_staircase(
    x: f32, y: f32, size: f32,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    let steps = 12;
    let step_angle = std::f32::consts::TAU / steps as f32;
    let rotation = time * 0.15;

    for i in 0..steps {
        let a1 = rotation + step_angle * i as f32;
        let a2 = rotation + step_angle * (i + 1) as f32;
        let r1 = size * 0.6;
        let r2 = size * 0.6 + size * 0.3 * ((i % 4) as f32 / 3.0);
        let x1 = x + r1 * a1.cos();
        let y1 = y + r1 * a1.sin();
        let x2 = x + r2 * a2.cos();
        let y2 = y + r2 * a2.sin();
        let brightness = 0.4 + 0.6 * ((i % 4) as f32 / 3.0);
        let alpha = 0.3 + 0.2 * ((i % 4) as f32 / 3.0);
        crate::engine::renderer::draw_line_world(
            x1, y1, x2, y2, cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r * brightness, color.g * brightness, color.b * brightness, alpha), 1.5);
        if i % 3 == 0 {
            let x3 = x + r2 * a1.cos();
            let y3 = y + r2 * a1.sin();
            crate::engine::renderer::draw_line_world(
                x1, y1, x3, y3, cam_x, cam_y, zoom, sw, sh,
                Color::new(color.r * brightness * 0.7, color.g * brightness * 0.7, color.b * brightness * 0.7, alpha * 0.8), 1.0);
        }
    }
    let pulse = (time * 2.5).sin() * 0.1 + 0.4;
    crate::engine::renderer::draw_circle_world(
        x, y, size * 0.06, cam_x, cam_y, zoom, sw, sh,
        Color::new(color.r, color.g, color.b, pulse));
}

/// Draw a non-Euclidean triangle angle sum visualization.
pub fn draw_angle_sum_demo(
    ax: f32, ay: f32, bx: f32, by: f32,
    cx_t: f32, cy_t: f32,
    angle_sum: f32, expected: f32,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color,
) {
    draw_geodesic_line(ax, ay, bx, by, cam_x, cam_y, zoom, sw, sh, color, 1.5, 10);
    draw_geodesic_line(bx, by, cx_t, cy_t, cam_x, cam_y, zoom, sw, sh, color, 1.5, 10);
    draw_geodesic_line(cx_t, cy_t, ax, ay, cam_x, cam_y, zoom, sw, sh, color, 1.5, 10);

    let mid_x = (ax + bx + cx_t) / 3.0;
    let mid_y = (ay + by + cy_t) / 3.0;
    let diff_color = if angle_sum < expected {
        Color::new(0.3, 0.8, 1.0, 0.8)
    } else {
        Color::new(1.0, 0.5, 0.2, 0.8)
    };
    crate::engine::renderer::draw_text_world(
        &format!("{:.0} deg", angle_sum), mid_x - 0.03, mid_y - 0.01,
        cam_x, cam_y, zoom, sw, sh, 0.04, diff_color);
}

/// Draw a hyperbolic tiling hint.
pub fn draw_hyperbolic_tiling_hint(
    center_x: f32, center_y: f32,
    radius: f32, depth: usize,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    let sides = 7;
    let tile_radius = radius * 0.35;
    draw_geodesic_ring(center_x, center_y, tile_radius, sides,
        cam_x, cam_y, zoom, sw, sh, color, 1.0, 6);

    if depth == 0 { return; }

    for i in 0..sides {
        let angle = std::f32::consts::TAU * i as f32 / sides as f32 + time * 0.05;
        let dist = tile_radius * 1.75;
        let tx = center_x + dist * angle.cos();
        let ty = center_y + dist * angle.sin();
        if tx * tx + ty * ty < 0.92 {
            let fade = Color::new(color.r * 0.6, color.g * 0.6, color.b * 0.6, color.a * 0.5);
            draw_geodesic_ring(tx, ty, tile_radius * 0.7, sides,
                cam_x, cam_y, zoom, sw, sh, fade, 0.7, 4);
        }
    }
}

/// Draw spherical great circle arcs.
/// On a sphere, the shortest path between two points is a great circle arc,
/// which appears as a curved line in the Mercator projection.
pub fn draw_great_circle_line(
    x1: f32, y1: f32, x2: f32, y2: f32,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color, thickness: f32,
    segments: usize,
) {
    // Convert to lat/lon
    let lat1 = y1 * std::f32::consts::FRAC_PI_2;
    let lon1 = x1 * std::f32::consts::PI;
    let lat2 = y2 * std::f32::consts::FRAC_PI_2;
    let lon2 = x2 * std::f32::consts::PI;

    // Interpolate along the great circle using slerp
    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;

        let (px1, py1) = slerp_latlon(lat1, lon1, lat2, lon2, t1);
        let (px2, py2) = slerp_latlon(lat1, lon1, lat2, lon2, t2);

        // Skip if the segment wraps around (large jump in x)
        if (px1 - px2).abs() > 0.5 { continue; }

        crate::engine::renderer::draw_line_world(
            px1, py1, px2, py2,
            cam_x, cam_y, zoom, sw, sh,
            color, thickness,
        );
    }
}

/// Spherical linear interpolation between two lat/lon points.
fn slerp_latlon(lat1: f32, lon1: f32, lat2: f32, lon2: f32, t: f32) -> (f32, f32) {
    // Convert to 3D unit vectors
    let v1 = [
        lat1.cos() * lon1.cos(),
        lat1.cos() * lon1.sin(),
        lat1.sin(),
    ];
    let v2 = [
        lat2.cos() * lon2.cos(),
        lat2.cos() * lon2.sin(),
        lat2.sin(),
    ];

    // Dot product
    let dot = v1[0]*v2[0] + v1[1]*v2[1] + v1[2]*v2[2];
    let dot = dot.clamp(-1.0, 1.0);
    let omega = dot.acos();

    if omega.abs() < 1e-6 {
        let lat = lat1 + (lat2 - lat1) * t;
        let lon = lon1 + (lon2 - lon1) * t;
        return (lon / std::f32::consts::PI, lat / std::f32::consts::FRAC_PI_2);
    }

    let sin_omega = omega.sin();
    let a = ((1.0 - t) * omega).sin() / sin_omega;
    let b = (t * omega).sin() / sin_omega;

    let ix = a * v1[0] + b * v2[0];
    let iy = a * v1[1] + b * v2[1];
    let iz = a * v1[2] + b * v2[2];

    let lat = iz.clamp(-1.0, 1.0).asin();
    let lon = iy.atan2(ix);

    (lon / std::f32::consts::PI, lat / std::f32::consts::FRAC_PI_2)
}

// ==================== ADVANCED ILLUSIONS (NEW) ====================

/// Draw an Escher-style recursive perspective grid that creates an
/// infinite-depth illusion using converging lines and diminishing tiles.
pub fn draw_escher_grid(
    cx: f32, cy: f32, size: f32, depth: usize,
    cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    let pulse = (time * 0.5).sin() * 0.1 + 1.0;
    draw_escher_recursive(cx, cy, size * pulse, depth, 0.0,
        cam_x, cam_y, zoom, sw, sh, color, time);
}

fn draw_escher_recursive(
    cx: f32, cy: f32, size: f32, depth: usize, angle_offset: f32,
    cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    if depth == 0 || size < 0.01 { return; }

    let half = size / 2.0;
    // Draw a diamond (rotated square)
    let corners = [
        (cx, cy - half),
        (cx + half, cy),
        (cx, cy + half),
        (cx - half, cy),
    ];

    let fade = depth as f32 / 6.0;
    let alpha = (color.a * fade).min(0.6);
    let c = Color::new(color.r, color.g, color.b, alpha);

    for i in 0..4 {
        let j = (i + 1) % 4;
        crate::engine::renderer::draw_line_world(
            corners[i].0, corners[i].1, corners[j].0, corners[j].1,
            cam_x, cam_y, zoom, sw, sh, c, 1.0,
        );
    }

    // Recurse into 4 sub-diamonds
    let sub_size = size * 0.45;
    let rot_speed = time * 0.1;
    let offsets = [
        (cx - half * 0.5, cy - half * 0.5),
        (cx + half * 0.5, cy - half * 0.5),
        (cx + half * 0.5, cy + half * 0.5),
        (cx - half * 0.5, cy + half * 0.5),
    ];

    for (ox, oy) in &offsets {
        draw_escher_recursive(*ox, *oy, sub_size, depth - 1, angle_offset + rot_speed,
            cam_x, cam_y, zoom, sw, sh, color, time);
    }
}

/// Draw a Klein bottle projection (impossible 4D surface).
/// The Klein bottle cannot exist in 3D without self-intersection.
pub fn draw_klein_bottle(
    cx: f32, cy: f32, size: f32,
    cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    let segments = 60;
    let tube_radius = size * 0.08;
    let rotation = time * 0.3;

    // Figure-8 immersion of the Klein bottle
    for i in 0..segments {
        let t1 = std::f32::consts::TAU * i as f32 / segments as f32;
        let t2 = std::f32::consts::TAU * (i + 1) as f32 / segments as f32;

        // Figure-8 curve in the xz plane
        let x1 = cx + size * 0.5 * ((1.0 + (t1 + rotation).sin()) * (t1 + rotation).cos()) * 0.5;
        let z1 = size * 0.3 * ((t1 + rotation).sin());
        let x2 = cx + size * 0.5 * ((1.0 + (t2 + rotation).sin()) * (t2 + rotation).cos()) * 0.5;
        let z2 = size * 0.3 * ((t2 + rotation).sin());

        // Tube cross-section
        let y1 = cy + size * 0.25 * (t1 + rotation).cos() * 0.5;
        let y2 = cy + size * 0.25 * (t2 + rotation).cos() * 0.5;

        // Depth-based coloring (simulate 3D by varying alpha)
        let depth1 = (z1 + size).abs() / (2.0 * size);
        let alpha = 0.15 + 0.35 * depth1;

        crate::engine::renderer::draw_line_world(
            x1, y1, x2, y2,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r, color.g, color.b, alpha), 1.5,
        );
    }

    // Cross-section circles at key points
    for k in 0..8 {
        let t = std::f32::consts::TAU * k as f32 / 8.0 + rotation;
        let bx = cx + size * 0.5 * ((1.0 + t.sin()) * t.cos()) * 0.5;
        let by = cy + size * 0.25 * t.cos() * 0.5;
        let cross_r = tube_radius * (1.0 + 0.3 * t.cos());
        crate::engine::renderer::draw_circle_outline_world(
            bx, by, cross_r,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r * 0.7, color.g * 0.7, color.b * 0.7, 0.2), 1.0,
        );
    }

    // Center glow
    let pulse = (time * 1.5).sin() * 0.15 + 0.25;
    crate::engine::renderer::draw_circle_world(
        cx, cy, size * 0.05, cam_x, cam_y, zoom, sw, sh,
        Color::new(color.r, color.g, color.b, pulse));
}

/// Draw a non-Euclidean mirror room illusion.
/// Shows parallel lines that appear to converge/diverge based on curvature.
pub fn draw_mirror_room(
    cx: f32, cy: f32, size: f32,
    cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32, curvature: f32,
) {
    let line_count = 16;
    let room_radius = size * 0.6;

    // Draw the "room" boundary
    crate::engine::renderer::draw_circle_outline_world(
        cx, cy, room_radius,
        cam_x, cam_y, zoom, sw, sh,
        Color::new(color.r * 0.5, color.g * 0.5, color.b * 0.5, 0.3), 1.5,
    );

    // Draw lines from center that curve based on curvature
    for i in 0..line_count {
        let base_angle = std::f32::consts::TAU * i as f32 / line_count as f32 + time * 0.02;
        let mut prev_x = cx;
        let mut prev_y = cy;

        for step in 1..=20 {
            let t = step as f32 / 20.0;
            let r = room_radius * t;

            // In positive curvature (spherical), lines converge
            // In negative curvature (hyperbolic), lines diverge
            let curvature_offset = curvature * t * t * 0.5;
            let angle = base_angle + curvature_offset;

            let px = cx + r * angle.cos();
            let py = cy + r * angle.sin();

            let alpha = (0.3 - t * 0.2).max(0.05);
            crate::engine::renderer::draw_line_world(
                prev_x, prev_y, px, py,
                cam_x, cam_y, zoom, sw, sh,
                Color::new(color.r, color.g, color.b, alpha), 1.0,
            );

            prev_x = px;
            prev_y = py;
        }
    }

    // "Mirror" reflections: concentric rings that get denser toward boundary
    for ring in 1..6 {
        let r = room_radius * ring as f32 / 6.0;
        let wave = (time * 2.0 + ring as f32 * 0.8).sin() * 0.005;
        let alpha = 0.08 + 0.04 * (time + ring as f32).sin();
        crate::engine::renderer::draw_circle_outline_world(
            cx, cy, r + wave,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r * 0.8, color.g * 0.8, color.b * 0.8, alpha), 0.8,
        );
    }
}

/// Draw a 5D penteract (5-cube) projected into 2D.
pub fn draw_penteract(
    cx: f32, cy: f32, size: f32,
    cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32,
    color: Color, time: f32,
) {
    // 32 vertices of a 5-cube
    let angle5d = time * 0.4;
    let angle4d = time * 0.25;
    let angle3d = time * 0.15;
    let c5 = angle5d.cos(); let s5 = angle5d.sin();
    let c4 = angle4d.cos(); let s4 = angle4d.sin();
    let c3 = angle3d.cos(); let s3 = angle3d.sin();

    let mut projected = Vec::new();
    for i in 0..32u32 {
        let x5 = if (i & 1) != 0 { 1.0 } else { -1.0 };
        let y5 = if (i & 2) != 0 { 1.0 } else { -1.0 };
        let z5 = if (i & 4) != 0 { 1.0 } else { -1.0 };
        let w5 = if (i & 8) != 0 { 1.0 } else { -1.0 };
        let v5 = if (i & 16) != 0 { 1.0 } else { -1.0 };

        // Rotate in xw plane
        let rx = x5 * c4 - w5 * s4;
        let rw = x5 * s4 + w5 * c4;
        // Rotate in yz plane
        let ry = y5 * c3 - z5 * s3;
        let rz = y5 * s3 + z5 * c3;
        // Rotate in xv plane (5D rotation)
        let rx2 = rx * c5 - v5 * s5;
        let rv = rx * s5 + v5 * c5;

        // Multi-step perspective projection: 5D -> 4D -> 3D -> 2D
        let pf1 = 2.5 / (2.5 - rv * 0.2);
        let x4 = rx2 * pf1;
        let y4 = ry * pf1;
        let z4 = rz * pf1;
        let w4 = rw * pf1;

        let pf2 = 2.5 / (2.5 - w4 * 0.2);
        let x3 = x4 * pf2;
        let y3 = y4 * pf2;
        let z3 = z4 * pf2;

        let pf3 = 3.0 / (3.0 - z3 * 0.3);
        let x2 = x3 * pf3;
        let y2 = y3 * pf3;

        projected.push((cx + x2 * size * 0.35, cy + y2 * size * 0.35));
    }

    // 80 edges of a 5-cube
    let edges: &[(usize, usize)] = &[
        // Cube 0 (v=0): 12 edges
        (0,1),(0,2),(0,4),(1,3),(1,5),(2,3),(2,6),(3,7),(4,5),(4,6),(5,7),(6,7),
        // Cube 1 (v=1): 12 edges
        (16,17),(16,18),(16,20),(17,19),(17,21),(18,19),(18,22),(19,23),(20,21),(20,22),(21,23),(22,23),
        // Connections between cube 0 and 1 (x-y-z same, v differs)
        (0,16),(1,17),(2,18),(3,19),(4,20),(5,21),(6,22),(7,23),
        // Cube 2 (w=1, v=0): 12 edges
        (8,9),(8,10),(8,12),(9,11),(9,13),(10,11),(10,14),(11,15),(12,13),(12,14),(13,15),(14,15),
        // Connections between cube 0 and 2 (w differs)
        (0,8),(1,9),(2,10),(3,11),(4,12),(5,13),(6,14),(7,15),
        // Cube 3 (w=1, v=1): 12 edges
        (24,25),(24,26),(24,28),(25,27),(25,29),(26,27),(26,30),(27,31),(28,29),(28,30),(29,31),(30,31),
        // Connections between cube 1 and 3
        (16,24),(17,25),(18,26),(19,27),(20,28),(21,29),(22,30),(23,31),
        // Connections between cube 2 and 3
        (8,24),(9,25),(10,26),(11,27),(12,28),(13,29),(14,30),(15,31),
    ];

    for &(a, b) in edges {
        let is_higher = (a as u32 >= 8 && b as u32 >= 8) && ((a ^ b) >= 8);
        let alpha = if is_higher { 0.1 + (time * 1.2).sin() * 0.05 } else { 0.35 };
        let thickness = if is_higher { 0.8 } else { 1.2 };
        crate::engine::renderer::draw_line_world(
            projected[a].0, projected[a].1, projected[b].0, projected[b].1,
            cam_x, cam_y, zoom, sw, sh,
            Color::new(color.r, color.g, color.b, alpha), thickness,
        );
    }

    // Center glow
    let pulse = (time * 1.5).sin() * 0.12 + 0.2;
    crate::engine::renderer::draw_circle_world(
        cx, cy, size * 0.06, cam_x, cam_y, zoom, sw, sh,
        Color::new(color.r, color.g, color.b, pulse));
}