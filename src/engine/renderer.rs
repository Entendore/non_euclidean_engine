use macroquad::prelude::*;
use crate::geometry::GeoPoint;

/// Helper rendering functions for the non-Euclidean engine.

pub fn draw_circle_world(
    cx: f32, cy: f32, radius: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color,
) {
    let half_w = screen_w * 0.45 * zoom;
    let half_h = screen_h * 0.45 * zoom;
    let sx = screen_w / 2.0 + (cx - camera_pos_x) * half_w;
    let sy = screen_h / 2.0 + (cy - camera_pos_y) * half_h;
    let sr = radius * half_w.min(half_h);
    if sr > 0.5 { draw_circle(sx, sy, sr, color); }
}

pub fn draw_circle_outline_world(
    cx: f32, cy: f32, radius: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color,
    line_width: f32,
) {
    let half_w = screen_w * 0.45 * zoom;
    let half_h = screen_h * 0.45 * zoom;
    let sx = screen_w / 2.0 + (cx - camera_pos_x) * half_w;
    let sy = screen_h / 2.0 + (cy - camera_pos_y) * half_h;
    let sr = radius * half_w.min(half_h);
    if sr > 0.5 { draw_circle_lines(sx, sy, sr, line_width, color); }
}

pub fn draw_line_world(
    x1: f32, y1: f32, x2: f32, y2: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color,
    thickness: f32,
) {
    let half_w = screen_w * 0.45 * zoom;
    let half_h = screen_h * 0.45 * zoom;
    let sx1 = screen_w / 2.0 + (x1 - camera_pos_x) * half_w;
    let sy1 = screen_h / 2.0 + (y1 - camera_pos_y) * half_h;
    let sx2 = screen_w / 2.0 + (x2 - camera_pos_x) * half_w;
    let sy2 = screen_h / 2.0 + (y2 - camera_pos_y) * half_h;
    draw_line(sx1, sy1, sx2, sy2, thickness, color);
}

pub fn draw_polygon_world(
    points: &[GeoPoint],
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color,
) {
    if points.len() < 3 { return; }
    let half_w = screen_w * 0.45 * zoom;
    let half_h = screen_h * 0.45 * zoom;
    let screen_pts: Vec<(f32, f32)> = points.iter().map(|p| {
        (screen_w / 2.0 + (p.x - camera_pos_x) * half_w,
         screen_h / 2.0 + (p.y - camera_pos_y) * half_h)
    }).collect();
    let cx: f32 = screen_pts.iter().map(|p| p.0).sum::<f32>() / screen_pts.len() as f32;
    let cy: f32 = screen_pts.iter().map(|p| p.1).sum::<f32>() / screen_pts.len() as f32;
    for i in 0..screen_pts.len() {
        let j = (i + 1) % screen_pts.len();
        draw_triangle(Vec2::new(cx, cy), Vec2::new(screen_pts[i].0, screen_pts[i].1), Vec2::new(screen_pts[j].0, screen_pts[j].1), color);
    }
}

pub fn draw_text_world(
    text: &str, x: f32, y: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    font_size: f32, color: Color,
) {
    let half_w = screen_w * 0.45 * zoom;
    let half_h = screen_h * 0.45 * zoom;
    let sx = screen_w / 2.0 + (x - camera_pos_x) * half_w;
    let sy = screen_h / 2.0 + (y - camera_pos_y) * half_h;
    let scaled_font = (font_size * zoom * half_w.min(half_h)).max(8.0).min(40.0);
    draw_text(text, sx, sy, scaled_font, color);
}

pub fn draw_arc_world(
    center_x: f32, center_y: f32, radius: f32,
    start_angle: f32, end_angle: f32, segments: usize,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, thickness: f32,
) {
    for i in 0..segments {
        let t1 = start_angle + (end_angle - start_angle) * i as f32 / segments as f32;
        let t2 = start_angle + (end_angle - start_angle) * (i + 1) as f32 / segments as f32;
        draw_line_world(
            center_x + radius * t1.cos(), center_y + radius * t1.sin(),
            center_x + radius * t2.cos(), center_y + radius * t2.sin(),
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            color, thickness,
        );
    }
}

// ======================== CONFORMAL SCALING (HYPERBOLIC) ========================

/// Draw a circle with Poincaré disk conformal scaling.
/// Objects near the edge of the disk appear smaller, objects near center appear larger.
/// This is the defining visual property of the Poincaré model.
pub fn draw_circle_world_conformal(
    cx: f32, cy: f32, radius: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color,
) {
    // Conformal factor: (1 - |p|^2) / 2
    let r2 = cx * cx + cy * cy;
    let conformal = if r2 < 1.0 { (1.0 - r2) / 2.0 } else { 0.01 };
    let effective_zoom = zoom * (conformal * 4.0).max(0.15);

    let half_w = screen_w * 0.45 * effective_zoom;
    let half_h = screen_h * 0.45 * effective_zoom;
    let sx = screen_w / 2.0 + (cx - camera_pos_x) * half_w;
    let sy = screen_h / 2.0 + (cy - camera_pos_y) * half_h;
    let sr = radius * half_w.min(half_h) * (conformal * 3.0).max(0.2);
    if sr > 0.3 { draw_circle(sx, sy, sr, color); }
}

/// Draw text with conformal scaling in the Poincaré disk.
pub fn draw_text_world_conformal(
    text: &str, x: f32, y: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    font_size: f32, color: Color,
) {
    let r2 = x * x + y * y;
    let conformal = if r2 < 1.0 { (1.0 - r2) / 2.0 } else { 0.01 };
    let effective_zoom = zoom * (conformal * 4.0).max(0.15);

    let half_w = screen_w * 0.45 * effective_zoom;
    let half_h = screen_h * 0.45 * effective_zoom;
    let sx = screen_w / 2.0 + (x - camera_pos_x) * half_w;
    let sy = screen_h / 2.0 + (y - camera_pos_y) * half_h;
    let scaled_font = (font_size * effective_zoom * half_w.min(half_h) * (conformal * 3.0).max(0.2)).max(6.0).min(36.0);
    draw_text(text, sx, sy, scaled_font, color);
}

/// Draw a circle outline with conformal scaling.
pub fn draw_circle_outline_world_conformal(
    cx: f32, cy: f32, radius: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, line_width: f32,
) {
    let r2 = cx * cx + cy * cy;
    let conformal = if r2 < 1.0 { (1.0 - r2) / 2.0 } else { 0.01 };
    let effective_zoom = zoom * (conformal * 4.0).max(0.15);

    let half_w = screen_w * 0.45 * effective_zoom;
    let half_h = screen_h * 0.45 * effective_zoom;
    let sx = screen_w / 2.0 + (cx - camera_pos_x) * half_w;
    let sy = screen_h / 2.0 + (cy - camera_pos_y) * half_h;
    let sr = radius * half_w.min(half_h) * (conformal * 3.0).max(0.2);
    if sr > 0.3 { draw_circle_lines(sx, sy, sr, line_width, color); }
}

// ======================== SPHERICAL CONFORMAL SCALING ========================

/// Draw a circle with spherical Mercator scaling.
/// Objects near the poles appear stretched/distorted.
pub fn draw_circle_world_spherical(
    cx: f32, cy: f32, radius: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color,
) {
    // Spherical conformal: cos(latitude) scaling
    let lat = cy * std::f32::consts::FRAC_PI_2;
    let cos_lat = lat.cos().max(0.15);
    let effective_zoom = zoom * cos_lat;

    let half_w = screen_w * 0.45 * effective_zoom;
    let half_h = screen_h * 0.45 * zoom; // vertical stays normal
    let sx = screen_w / 2.0 + (cx - camera_pos_x) * half_w;
    let sy = screen_h / 2.0 + (cy - camera_pos_y) * half_h;
    // Horizontally compressed near poles, vertically normal
    let sr_x = radius * half_w;
    let sr = sr_x.min(radius * half_h);
    if sr > 0.3 { draw_circle(sx, sy, sr, color); }
}

// ======================== PORTAL GEODESIC CONNECTIONS ========================

/// Draw a geodesic arc connecting two portal endpoints, showing the "wormhole" link.
/// Uses a smooth curved path (cubic bezier) with dashed animation.
pub fn draw_portal_geodesic_link(
    ax: f32, ay: f32, bx: f32, by: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, thickness: f32,
    time: f32, segments: usize,
) {
    // Control points for a smooth curve (bow outward from midpoint)
    let _mx = (ax + bx) / 2.0;
    let _my = (ay + by) / 2.0;
    let dx = bx - ax;
    let dy = by - ay;
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    // Perpendicular offset for curve
    let perp_x = -dy / len;
    let perp_y = dx / len;
    let bow = len * 0.3;
    let c1x = ax + dx * 0.25 + perp_x * bow;
    let c1y = ay + dy * 0.25 + perp_y * bow;
    let c2x = ax + dx * 0.75 + perp_x * bow;
    let c2y = ay + dy * 0.75 + perp_y * bow;

    // Animated dash pattern
    let dash_offset = time * 2.0;
    let dash_len = 3.0_f32; // segments per dash

    for i in 0..segments {
        let t = i as f32 / segments as f32;
        let t_next = (i + 1) as f32 / segments as f32;

        // Bezier interpolation
        let px = bezier(t, ax, c1x, c2x, bx);
        let py = bezier(t, ay, c1y, c2y, by);
        let nx = bezier(t_next, ax, c1x, c2x, bx);
        let ny = bezier(t_next, ay, c1y, c2y, by);

        // Dash pattern
        let dash_phase = ((i as f32 + dash_offset) % (dash_len * 2.0)) / dash_len;
        let alpha = if dash_phase < 1.0 { 0.6 + 0.3 * (1.0 - dash_phase) } else { 0.1 };

        draw_line_world(px, py, nx, ny, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(color.r, color.g, color.b, alpha), thickness);
    }

    // Traveling particle along the link
    let particle_t = (time * 0.5) % 1.0;
    let ppx = bezier(particle_t, ax, c1x, c2x, bx);
    let ppy = bezier(particle_t, ay, c1y, c2y, by);
    draw_circle_world(ppx, ppy, 0.012, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
        Color::new(color.r, color.g, color.b, 0.8));

    // Second particle going the other way
    let particle_t2 = (time * 0.5 + 0.5) % 1.0;
    let ppx2 = bezier(particle_t2, ax, c1x, c2x, bx);
    let ppy2 = bezier(particle_t2, ay, c1y, c2y, by);
    draw_circle_world(ppx2, ppy2, 0.010, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
        Color::new(color.r, color.g, color.b, 0.6));
}

fn bezier(t: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

// ======================== GRAVITY LENS EFFECT ========================

/// Draw a gravitational lensing effect around a point.
/// Concentric rings that compress toward the center, simulating
/// spacetime curvature. Used around portals and high-curvature objects.
pub fn draw_gravity_lens(
    cx: f32, cy: f32, strength: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, time: f32,
) {
    let ring_count = 6;
    for i in 0..ring_count {
        let t = i as f32 / ring_count as f32;
        // Exponential compression toward center
        let ring_r = strength * (1.0 - t * t * t);
        if ring_r < 0.003 { continue; }

        let wobble = (time * 3.0 + i as f32 * 1.1).sin() * 0.002;
        let alpha = (0.25 + 0.3 * (1.0 - t)) * (0.8 + 0.2 * (time * 2.0 + i as f32 * 0.5).sin());

        draw_circle_outline_world(
            cx, cy, ring_r + wobble,
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(color.r, color.g, color.b, alpha),
            1.5 - t * 0.8,
        );
    }

    // Distortion rays
    let ray_count = 8;
    for i in 0..ray_count {
        let angle = std::f32::consts::TAU * i as f32 / ray_count as f32 + time * 0.3;
        let inner_r = strength * 0.15;
        let outer_r = strength * (0.6 + 0.15 * (time * 2.0 + i as f32).sin());
        let x1 = cx + inner_r * angle.cos();
        let y1 = cy + inner_r * angle.sin();
        let x2 = cx + outer_r * angle.cos();
        let y2 = cy + outer_r * angle.sin();
        draw_line_world(x1, y1, x2, y2, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(color.r, color.g, color.b, 0.12), 0.8);
    }
}

// ======================== SIZE ILLUSION DEMO ========================

/// Draw a pair of identical-size circles at different positions to demonstrate
/// how the same object appears different sizes in non-Euclidean geometry.
/// In the Poincaré disk, objects near the edge appear much smaller.
pub fn draw_size_illusion_pair(
    ax: f32, ay: f32, bx: f32, by: f32,
    base_radius: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, label_a: &str, label_b: &str,
    use_conformal: bool,
) {
    if use_conformal {
        // Both drawn with conformal scaling — they'll look different sizes!
        draw_circle_world_conformal(ax, ay, base_radius, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, color);
        draw_circle_outline_world_conformal(ax, ay, base_radius, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(1.0, 1.0, 1.0, 0.3), 1.0);
        draw_text_world_conformal(label_a, ax - 0.015, ay - base_radius - 0.01, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, 0.02, WHITE);

        draw_circle_world_conformal(bx, by, base_radius, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, color);
        draw_circle_outline_world_conformal(bx, by, base_radius, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(1.0, 1.0, 0.0, 0.4), 1.0);
        draw_text_world_conformal(label_b, bx - 0.015, by - base_radius - 0.01, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, 0.02, YELLOW);
    } else {
        draw_circle_world(ax, ay, base_radius, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, color);
        draw_text_world(label_a, ax - 0.015, ay - base_radius - 0.01, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, 0.03, WHITE);
        draw_circle_world(bx, by, base_radius, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, color);
        draw_text_world(label_b, bx - 0.015, by - base_radius - 0.01, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, 0.03, YELLOW);
    }

    // "Same size!" label between them
    let mx = (ax + bx) / 2.0;
    let my = (ay + by) / 2.0 - base_radius - 0.03;
    draw_text_world("same hyperbolic size!", mx - 0.04, my,
        camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, 0.02,
        Color::new(0.7, 0.7, 0.8, 0.5 + (get_time() as f32 * 1.5).sin() * 0.2));
}

// ======================== ON-SCREEN HELPERS ========================

/// Draw an on-screen arrow pointing from (from_x, from_y) toward (to_x, to_y).
pub fn draw_screen_arrow(from_x: f32, from_y: f32, to_x: f32, to_y: f32, color: Color, size: f32) {
    let dx = to_x - from_x;
    let dy = to_y - from_y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1.0 { return; }
    let angle = dy.atan2(dx);

    let tip_x = from_x + (dx / dist) * size;
    let tip_y = from_y + (dy / dist) * size;
    let left_x = tip_x - size * 0.5 * (angle + 2.5).cos();
    let left_y = tip_y - size * 0.5 * (angle + 2.5).sin();
    let right_x = tip_x - size * 0.5 * (angle - 2.5).cos();
    let right_y = tip_y - size * 0.5 * (angle - 2.5).sin();

    draw_triangle(
        Vec2::new(tip_x, tip_y),
        Vec2::new(left_x, left_y),
        Vec2::new(right_x, right_y),
        color,
    );
}

/// Draw the proximity indicator.
pub fn draw_proximity_indicator(
    _player_sx: f32, _player_sy: f32,
    target_sx: f32, target_sy: f32,
    screen_w: f32, screen_h: f32,
    color: Color, label: &str,
) {
    let margin = 60.0;
    if target_sx > margin && target_sx < screen_w - margin
        && target_sy > margin && target_sy < screen_h - margin {
        return;
    }
    let cx = screen_w / 2.0;
    let cy = screen_h / 2.0;
    let dx = target_sx - cx;
    let dy = target_sy - cy;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1.0 { return; }
    let edge_x = cx + (dx / dist) * (screen_w / 2.0 - 40.0);
    let edge_y = cy + (dy / dist) * (screen_h / 2.0 - 40.0);
    let ex = edge_x.clamp(25.0, screen_w - 25.0);
    let ey = edge_y.clamp(95.0, screen_h - 25.0);

    let pulse = (get_time() * 4.0).sin() * 0.3 + 0.7;
    draw_circle(ex, ey, 12.0, Color::new(color.r, color.g, color.b, (0.3 * pulse) as f32));
    draw_circle(ex, ey, 6.0, color);
    draw_screen_arrow(ex, ey, ex + dx / dist * 8.0, ey + dy / dist * 8.0, color, 8.0);
    draw_text(label, ex - 4.0, ey - 18.0, 14.0, color);
}

/// Draw a progress bar.
pub fn draw_progress_bar(x: f32, y: f32, w: f32, h: f32, fraction: f32, bg_color: Color, fill_color: Color, border_color: Color) {
    draw_rectangle(x, y, w, h, bg_color);
    let fill_w = (w * fraction).max(0.0);
    if fill_w > 0.0 { draw_rectangle(x, y, fill_w, h, fill_color); }
    draw_rectangle_lines(x, y, w, h, 1.5, border_color);
}

/// Draw a minimap.
pub fn draw_minimap(
    player_x: f32, player_y: f32,
    collectibles: &[(f32, f32, bool, char)],
    enemies: &[(f32, f32)],
    world_bounds: (f32, f32, f32, f32),
    size: f32,
    screen_w: f32, screen_h: f32,
    geometry_type: &str,
) {
    let mx = screen_w - size - 15.0;
    let my = screen_h - size - 15.0;
    draw_rectangle(mx, my, size, size, Color::new(0.0, 0.0, 0.0, 0.6));
    draw_rectangle_lines(mx, my, size, size, 1.0, Color::new(0.4, 0.4, 0.5, 0.7));
    let (wx0, wy0, wx1, wy1) = world_bounds;
    let ww = wx1 - wx0;
    let wh = wy1 - wy0;
    if ww < 0.001 || wh < 0.001 { return; }
    let to_map = |wx: f32, wy: f32| -> (f32, f32) {
        (mx + ((wx - wx0) / ww) * size, my + ((wy - wy0) / wh) * size)
    };
    for &(cx, cy, collected, _label) in collectibles {
        let (px, py) = to_map(cx, cy);
        if px >= mx && px <= mx + size && py >= my && py <= my + size {
            let c = if collected { Color::new(0.3, 0.3, 0.3, 0.5) } else { YELLOW };
            draw_circle(px, py, 2.5, c);
        }
    }
    for &(ex, ey) in enemies {
        let (px, py) = to_map(ex, ey);
        if px >= mx && px <= mx + size && py >= my && py <= my + size {
            draw_circle(px, py, 2.0, RED);
        }
    }
    let (ppx, ppy) = to_map(player_x, player_y);
    draw_circle(ppx, ppy, 3.5, GREEN);
    draw_text(geometry_type, mx + 3.0, my + 2.0, 10.0, Color::new(0.5, 0.5, 0.6, 0.7));
}

/// Draw the dash energy bar.
pub fn draw_dash_bar(x: f32, y: f32, energy: f32, is_dashing: bool) {
    let w = 100.0;
    let h = 8.0;
    draw_rectangle(x, y, w, h, Color::new(0.1, 0.1, 0.1, 0.6));
    let fill_w = w * energy;
    let color = if is_dashing { Color::new(0.3, 0.8, 1.0, 0.9) }
        else if energy > 0.3 { Color::new(0.2, 0.5, 0.8, 0.7) }
        else { Color::new(0.5, 0.2, 0.2, 0.7) };
    if fill_w > 0.0 { draw_rectangle(x, y, fill_w, h, color); }
    draw_rectangle_lines(x, y, w, h, 1.0, Color::new(0.4, 0.4, 0.5, 0.5));
    draw_text("DASH [Shift]", x, y - 14.0, 11.0, Color::new(0.4, 0.4, 0.5, 0.6));
}

/// Draw combo display.
pub fn draw_combo(x: f32, y: f32, combo: u32, combo_timer: f32, display_text: &str, display_timer: f32) {
    if combo > 1 && combo_timer > 0.0 {
        let alpha = combo_timer.min(1.0);
        let scale = 1.0 + (combo as f32 * 0.1).min(0.5);
        let size = (20.0 * scale).min(36.0);
        draw_text(&format!("x{} COMBO", combo), x, y, size, Color::new(1.0, 0.8, 0.0, alpha));
    }
    if display_timer > 0.0 {
        let alpha = display_timer.min(1.0);
        let float_y = y - 30.0 - (1.5 - display_timer) * 20.0;
        draw_text(display_text, x, float_y, 16.0, Color::new(1.0, 1.0, 0.5, alpha));
    }
}

/// Draw player trail.
pub fn draw_trail(
    trail: &[(f32, f32, f32)],
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
    color: Color,
) {
    for &(tx, ty, alpha) in trail {
        if alpha > 0.0 {
            draw_circle_world(tx, ty, 0.02 * alpha, cam_x, cam_y, zoom, sw, sh,
                Color::new(color.r, color.g, color.b, alpha * 0.5));
        }
    }
}

/// Draw an enemy.
pub fn draw_enemy(
    e: &crate::engine::game::Enemy,
    cam_x: f32, cam_y: f32,
    zoom: f32, sw: f32, sh: f32,
) {
    let pulse = (e.phase).sin() * 0.005;
    draw_circle_world(e.pos.x, e.pos.y, e.alert_radius, cam_x, cam_y, zoom, sw, sh,
        Color::new(1.0, 0.0, 0.0, 0.06));
    draw_circle_world(e.pos.x, e.pos.y, e.radius + 0.015 + pulse, cam_x, cam_y, zoom, sw, sh,
        Color::new(1.0, 0.1, 0.0, 0.15));
    draw_circle_world(e.pos.x, e.pos.y, e.radius + pulse, cam_x, cam_y, zoom, sw, sh, e.color);
    draw_circle_world(e.pos.x, e.pos.y, e.radius * 0.35, cam_x, cam_y, zoom, sw, sh,
        Color::new(1.0, 1.0, 0.5, 0.9));
}

// ======================== HYPERSPACE RENDERING (4D/5D) ========================

/// Draw a circle with hyperspace projection scaling.
/// Uses a custom scale factor that simulates the distortion from
/// projecting higher-dimensional spaces down to 2D.
pub fn draw_circle_world_hyperspace(
    cx: f32, cy: f32, radius: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color,
    hyperspace_scale: f32,
    extra_dims_offset: f32,
) {
    let effective_zoom = zoom * hyperspace_scale.max(0.1);
    let half_w = screen_w * 0.45 * effective_zoom;
    let half_h = screen_h * 0.45 * effective_zoom;
    let sx = screen_w / 2.0 + (cx - camera_pos_x + extra_dims_offset) * half_w;
    let sy = screen_h / 2.0 + (cy - camera_pos_y) * half_h;
    let sr = radius * half_w.min(half_h) * hyperspace_scale.max(0.2);
    if sr > 0.3 { draw_circle(sx, sy, sr, color); }
}

/// Draw a circle outline with hyperspace projection scaling.
pub fn draw_circle_outline_world_hyperspace(
    cx: f32, cy: f32, radius: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, line_width: f32,
    hyperspace_scale: f32,
    extra_dims_offset: f32,
) {
    let effective_zoom = zoom * hyperspace_scale.max(0.1);
    let half_w = screen_w * 0.45 * effective_zoom;
    let half_h = screen_h * 0.45 * effective_zoom;
    let sx = screen_w / 2.0 + (cx - camera_pos_x + extra_dims_offset) * half_w;
    let sy = screen_h / 2.0 + (cy - camera_pos_y) * half_h;
    let sr = radius * half_w.min(half_h) * hyperspace_scale.max(0.2);
    if sr > 0.3 { draw_circle_lines(sx, sy, sr, line_width, color); }
}

/// Draw text with hyperspace projection scaling.
pub fn draw_text_world_hyperspace(
    text: &str, x: f32, y: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    font_size: f32, color: Color,
    hyperspace_scale: f32,
    extra_dims_offset: f32,
) {
    let effective_zoom = zoom * hyperspace_scale.max(0.1);
    let half_w = screen_w * 0.45 * effective_zoom;
    let half_h = screen_h * 0.45 * effective_zoom;
    let sx = screen_w / 2.0 + (x - camera_pos_x + extra_dims_offset) * half_w;
    let sy = screen_h / 2.0 + (y - camera_pos_y) * half_h;
    let scaled_font = (font_size * effective_zoom * half_w.min(half_h) * hyperspace_scale.max(0.2)).max(6.0).min(36.0);
    draw_text(text, sx, sy, scaled_font, color);
}

/// Draw a "ghost" of an object in a nearby parallel dimension.
/// Renders a semi-transparent copy offset by the extra-dimensional distance.
pub fn draw_dimensional_ghost(
    cx: f32, cy: f32, radius: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color,
    dim_offset_x: f32, dim_offset_y: f32,
    ghost_alpha: f32,
) {
    let half_w = screen_w * 0.45 * zoom;
    let half_h = screen_h * 0.45 * zoom;
    let sx = screen_w / 2.0 + (cx + dim_offset_x - camera_pos_x) * half_w;
    let sy = screen_h / 2.0 + (cy + dim_offset_y - camera_pos_y) * half_h;
    let sr = radius * half_w.min(half_h);
    if sr > 0.5 {
        draw_circle(sx, sy, sr, Color::new(color.r, color.g, color.b, ghost_alpha));
        draw_circle_lines(sx, sy, sr, 1.0, Color::new(color.r, color.g, color.b, ghost_alpha * 0.5));
    }
}

/// Draw a wireframe tesseract (4D hypercube) projection.
/// Shows the classic "cube within a cube" visualization.
pub fn draw_tesseract_wireframe(
    cx: f32, cy: f32, size: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, rotation: f32,
) {
    // 4D rotation angle (simplified projection)
    let r4d = rotation * 0.7;
    let r3d = rotation * 0.5;

    // Inner and outer cubes represent the 4D hypercube
    let inner_scale = 0.4 + 0.1 * r4d.cos();
    let skew_x = 0.15 * r4d.sin();
    let skew_y = 0.15 * r4d.cos();

    // 8 vertices of outer cube
    let outer: [(f32, f32); 8] = [
        (-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0),
        (-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0),
    ];
    // 8 vertices of inner cube (shifted by 4D component)
    let inner: [(f32, f32); 8] = [
        (-inner_scale + skew_x, -inner_scale + skew_y),
        (inner_scale + skew_x, -inner_scale + skew_y),
        (inner_scale + skew_x, inner_scale + skew_y),
        (-inner_scale + skew_x, inner_scale + skew_y),
        (-inner_scale + skew_x, -inner_scale + skew_y),
        (inner_scale + skew_x, -inner_scale + skew_y),
        (inner_scale + skew_x, inner_scale + skew_y),
        (-inner_scale + skew_x, inner_scale + skew_y),
    ];

    // Draw edges: outer cube
    let edges = [(0,1),(1,2),(2,3),(3,0),(4,5),(5,6),(6,7),(7,4),(0,4),(1,5),(2,6),(3,7)];
    let alpha = 0.3 + 0.1 * r3d.sin();
    for &(i, j) in &edges[0..4] {
        draw_line_world(
            cx + outer[i].0 * size, cy + outer[i].1 * size,
            cx + outer[j].0 * size, cy + outer[j].1 * size,
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(color.r, color.g, color.b, alpha), 1.5);
    }
    // Inner cube
    for &(i, j) in &edges[0..4] {
        draw_line_world(
            cx + inner[i].0 * size, cy + inner[i].1 * size,
            cx + inner[j].0 * size, cy + inner[j].1 * size,
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(color.r, color.g, color.b, alpha * 0.8), 1.0);
    }
    // Connecting edges (4D dimension)
    for i in 0..4 {
        draw_line_world(
            cx + outer[i].0 * size, cy + outer[i].1 * size,
            cx + inner[i].0 * size, cy + inner[i].1 * size,
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(color.r, color.g, color.b, alpha * 0.5), 0.8);
    }
}

/// Draw a 5D hypercube (penteract) wireframe projection.
pub fn draw_penteract_wireframe(
    cx: f32, cy: f32, size: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, rotation: f32,
) {
    // Three nested squares representing the 5D hypercube
    let scales = [
        1.0,
        0.6 + 0.15 * (rotation * 0.6).cos(),
        0.25 + 0.1 * (rotation * 0.9).sin(),
    ];
    let offsets = [
        (0.0, 0.0),
        (0.12 * (rotation * 0.7).sin(), 0.12 * (rotation * 0.5).cos()),
        (0.2 * (rotation * 0.4).cos(), -0.15 * (rotation * 0.8).sin()),
    ];

    for layer in 0..3 {
        let s = scales[layer];
        let ox = offsets[layer].0;
        let oy = offsets[layer].1;
        let alpha = (0.25 - layer as f32 * 0.05).max(0.1);

        // Square edges
        let corners = [
            (cx + (-s + ox) * size, cy + (-s + oy) * size),
            (cx + (s + ox) * size, cy + (-s + oy) * size),
            (cx + (s + ox) * size, cy + (s + oy) * size),
            (cx + (-s + ox) * size, cy + (s + oy) * size),
        ];
        for i in 0..4 {
            let j = (i + 1) % 4;
            draw_line_world(
                corners[i].0, corners[i].1,
                corners[j].0, corners[j].1,
                camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
                Color::new(color.r, color.g, color.b, alpha), 1.2 - layer as f32 * 0.3);
        }
        // Connect to next layer
        if layer < 2 {
            for i in 0..4 {
                let next_s = scales[layer + 1];
                let next_ox = offsets[layer + 1].0;
                let next_oy = offsets[layer + 1].1;
                let next_corners = [
                    (cx + (-next_s + next_ox) * size, cy + (-next_s + next_oy) * size),
                    (cx + (next_s + next_ox) * size, cy + (-next_s + next_oy) * size),
                    (cx + (next_s + next_ox) * size, cy + (next_s + next_oy) * size),
                    (cx + (-next_s + next_ox) * size, cy + (next_s + next_oy) * size),
                ];
                draw_line_world(
                    corners[i].0, corners[i].1,
                    next_corners[i].0, next_corners[i].1,
                    camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
                    Color::new(color.r, color.g, color.b, alpha * 0.4), 0.7);
            }
        }
    }
}

/// Draw a hypersphere grid visualization showing the extra dimensions.
pub fn draw_hyperspace_grid(
    center_x: f32, center_y: f32, radius: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, time: f32, dimensions: usize,
) {
    // Concentric rings representing "dimension slices"
    let ring_count = if dimensions == 5 { 8 } else { 6 };
    for i in 1..=ring_count {
        let r = radius * i as f32 / ring_count as f32;
        let wobble = (time * 2.0 + i as f32 * 0.5).sin() * 0.003;
        let alpha = (0.15 + 0.1 * (1.0 - i as f32 / ring_count as f32))
            * (0.7 + 0.3 * (time + i as f32 * 0.3).sin());
        draw_circle_outline_world(
            center_x, center_y, r + wobble,
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(color.r, color.g, color.b, alpha), 0.8);
    }

    // Radial lines representing dimension axes
    let axis_count = dimensions + 2;
    for i in 0..axis_count {
        let angle = std::f32::consts::TAU * i as f32 / axis_count as f32 + time * 0.2;
        let extra_wobble = if i >= 4 {
            // Extra dimensions wobble more
            (time * 3.0 + i as f32).sin() * 0.05
        } else {
            0.0
        };
        draw_line_world(
            center_x, center_y,
            center_x + (radius + extra_wobble) * angle.cos(),
            center_y + (radius + extra_wobble) * angle.sin(),
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(color.r, color.g, color.b, 0.12), 0.6);
    }

    // Dimension labels
    let dim_labels = ["X", "Y", "Z", "W", "V"];
    for i in 0..dimensions.min(5) {
        let angle = std::f32::consts::TAU * i as f32 / dimensions as f32 + time * 0.2;
        let label_r = radius + 0.08;
        draw_text_world(
            dim_labels[i],
            center_x + label_r * angle.cos() - 0.01,
            center_y + label_r * angle.sin() - 0.01,
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            0.02,
            if i >= 2 {
                Color::new(1.0, 0.5, 0.0, 0.6 + (time * 2.0 + i as f32).sin() * 0.2)
            } else {
                Color::new(color.r, color.g, color.b, 0.5)
            });
    }
}

/// Draw a dimension shift visual effect (transition between 4D and 5D).
pub fn draw_dimension_shift_effect(
    screen_w: f32, screen_h: f32,
    progress: f32, color: Color, time: f32,
) {
    // Rotating dimensional rift lines
    let line_count = 12;
    for i in 0..line_count {
        let angle = std::f32::consts::TAU * i as f32 / line_count as f32 + time * 3.0;
        let inner_r = 50.0 * progress;
        let outer_r = 200.0 + 100.0 * (time * 2.0 + i as f32 * 0.5).sin();
        let x1 = screen_w / 2.0 + inner_r * angle.cos();
        let y1 = screen_h / 2.0 + inner_r * angle.sin();
        let x2 = screen_w / 2.0 + outer_r * angle.cos();
        let y2 = screen_h / 2.0 + outer_r * angle.sin();
        let alpha = progress * (0.2 + 0.1 * (time * 3.0 + i as f32).sin());
        draw_line(x1, y1, x2, y2, 1.0, Color::new(color.r, color.g, color.b, alpha));
    }
}

// ==================== NEW: Enhanced Rendering Functions ====================

/// Draw the Poincaré disk boundary circle with animated glow.
pub fn draw_poincare_boundary(
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, time: f32,
) {
    // Animated boundary glow
    let segments = 80;
    let half_w = screen_w * 0.45 * zoom;
    let half_h = screen_h * 0.45 * zoom;

    for i in 0..segments {
        let angle1 = std::f32::consts::TAU * i as f32 / segments as f32;
        let angle2 = std::f32::consts::TAU * (i + 1) as f32 / segments as f32;

        let sx1 = screen_w / 2.0 + (angle1.cos() * 0.999 - camera_pos_x) * half_w;
        let sy1 = screen_h / 2.0 + (angle1.sin() * 0.999 - camera_pos_y) * half_h;
        let sx2 = screen_w / 2.0 + (angle2.cos() * 0.999 - camera_pos_x) * half_w;
        let sy2 = screen_h / 2.0 + (angle2.sin() * 0.999 - camera_pos_y) * half_h;

        // Pulsing brightness along the boundary
        let pulse = (time * 3.0 + i as f32 * 0.3).sin() * 0.2 + 0.8;
        let alpha = color.a * pulse;
        draw_line(sx1, sy1, sx2, sy2, 2.0, Color::new(color.r, color.g, color.b, alpha));
    }

    // Outer glow ring
    for i in 0..segments {
        let angle = std::f32::consts::TAU * i as f32 / segments as f32;
        let sx = screen_w / 2.0 + (angle.cos() * 1.01 - camera_pos_x) * half_w;
        let sy = screen_h / 2.0 + (angle.sin() * 1.01 - camera_pos_y) * half_h;
        let pulse = (time * 2.0 + i as f32 * 0.5).sin() * 0.15 + 0.15;
        draw_circle(sx, sy, 3.0, Color::new(color.r, color.g, color.b, pulse));
    }
}

/// Draw Klein model geodesics (straight lines) for comparison with Poincaré arcs.
pub fn draw_klein_geodesic(
    x1: f32, y1: f32, x2: f32, y2: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, thickness: f32,
) {
    draw_line_world(x1, y1, x2, y2, camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, color, thickness);
}

/// Draw a horocycle in the Poincaré disk.
pub fn draw_horocycle(
    ideal_angle: f32, offset: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, thickness: f32, time: f32,
) {
    let ix = ideal_angle.cos();
    let iy = ideal_angle.sin();
    let effective_r = offset.clamp(0.05, 0.9);
    let center_dist = 1.0 - effective_r;
    let cx = ix * center_dist;
    let cy = iy * center_dist;

    let segments = 48;
    for i in 0..segments {
        let a1 = std::f32::consts::TAU * i as f32 / segments as f32;
        let a2 = std::f32::consts::TAU * (i + 1) as f32 / segments as f32;

        let px1 = cx + effective_r * a1.cos();
        let py1 = cy + effective_r * a1.sin();
        let px2 = cx + effective_r * a2.cos();
        let py2 = cy + effective_r * a2.sin();

        // Only draw if inside disk
        if px1 * px1 + py1 * py1 < 0.99 && px2 * px2 + py2 * py2 < 0.99 {
            let pulse = (time * 2.0 + i as f32 * 0.2).sin() * 0.15 + 0.85;
            draw_line_world(
                px1, py1, px2, py2,
                camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
                Color::new(color.r, color.g, color.b, color.a * pulse),
                thickness,
            );
        }
    }

    // Mark the ideal point
    draw_circle_world(
        ix * 0.99, iy * 0.99, 0.012,
        camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
        Color::new(color.r, color.g, color.b, 0.5 + (time * 3.0).sin() * 0.3),
    );
}

/// Draw ideal points on the Poincaré disk boundary.
pub fn draw_ideal_points(
    count: usize,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, time: f32,
) {
    for i in 0..count {
        let angle = std::f32::consts::TAU * i as f32 / count as f32;
        let px = angle.cos() * 0.995;
        let py = angle.sin() * 0.995;
        let pulse = (time * 2.0 + i as f32 * 0.8).sin() * 0.3 + 0.7;
        let size = 0.008 + (time * 1.5 + i as f32).sin() * 0.003;

        draw_circle_world(px, py, size,
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(color.r, color.g, color.b, color.a * pulse));

        // Tiny label
        draw_text_world(
            &format!("inf{}", i + 1),
            px * 1.02 - 0.015, py * 1.02 - 0.01,
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            0.015,
            Color::new(color.r, color.g, color.b, 0.3 * pulse),
        );
    }
}

/// Draw a portal preview window — shows what's near the exit of a portal.
pub fn draw_portal_preview(
    portal_ax: f32, portal_ay: f32, portal_bx: f32, portal_by: f32,
    radius: f32, time: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    exit_color: Color,
) {
    let half_w = screen_w * 0.45 * zoom;
    let half_h = screen_h * 0.45 * zoom;

    // Draw a small "window" near portal A that shows a view toward portal B
    let preview_offset = radius * 1.8;
    let angle = (portal_by - portal_ay).atan2(portal_bx - portal_ax);
    let wx = portal_ax + preview_offset * angle.cos();
    let wy = portal_ay + preview_offset * angle.sin();

    let sx = screen_w / 2.0 + (wx - camera_pos_x) * half_w;
    let sy = screen_h / 2.0 + (wy - camera_pos_y) * half_h;
    let sr = radius * half_w.min(half_h) * 0.5;

    if sr < 5.0 { return; }

    // Pulsing border
    let pulse = (time * 3.0).sin() * 0.2 + 0.6;

    // Dark background
    draw_circle(sx, sy, sr, Color::new(0.0, 0.0, 0.0, 0.5 * pulse));
    // Border
    draw_circle_lines(sx, sy, sr, 1.5, Color::new(exit_color.r, exit_color.g, exit_color.b, 0.5 * pulse));

    // Mini arrow pointing toward exit
    let arrow_angle = angle;
    let arrow_len = sr * 0.5;
    let ax1 = sx;
    let ay1 = sy;
    let ax2 = sx + arrow_len * arrow_angle.cos();
    let ay2 = sy + arrow_len * arrow_angle.sin();
    draw_line(ax1, ay1, ax2, ay2, 1.5, Color::new(exit_color.r, exit_color.g, exit_color.b, 0.6 * pulse));
}

/// Draw portal cooldown indicator (circular progress around the portal).
pub fn draw_portal_cooldown(
    px: f32, py: f32, radius: f32, cooldown: f32, max_cooldown: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
) {
    if cooldown <= 0.0 { return; }
    let progress = 1.0 - cooldown / max_cooldown;
    if progress >= 1.0 { return; }

    let segments = 16;
    let half_w = screen_w * 0.45 * zoom;
    let half_h = screen_h * 0.45 * zoom;
    let scx = screen_w / 2.0 + (px - camera_pos_x) * half_w;
    let scy = screen_h / 2.0 + (py - camera_pos_y) * half_h;
    let sr = (radius + 0.015) * half_w.min(half_h);

    let fill_segments = (segments as f32 * progress) as usize;
    for i in 0..fill_segments {
        let a1 = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * i as f32 / segments as f32;
        let a2 = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * (i + 1) as f32 / segments as f32;
        draw_line(
            scx + sr * a1.cos(), scy + sr * a1.sin(),
            scx + sr * a2.cos(), scy + sr * a2.sin(),
            2.0, Color::new(0.5, 0.5, 0.5, 0.4),
        );
    }
}

/// Draw a cross-section visualization for 4D/5D geometry.
/// Shows a 2D "slice" of the higher-dimensional space at a given coordinate.
pub fn draw_cross_section(
    center_x: f32, center_y: f32, radius: f32,
    slice_coord: f32, slice_axis: usize,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color, time: f32,
) {
    // Draw the cross-section circle/ellipse
    let pulse = (time * 1.5).sin() * 0.005;
    draw_circle_outline_world(
        center_x, center_y, radius + pulse,
        camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
        Color::new(color.r, color.g, color.b, 0.3), 1.5,
    );

    // Axis indicator for the slice dimension
    let axis_labels = ["X", "Y", "Z", "W", "V"];
    let label = if slice_axis < 5 { axis_labels[slice_axis] } else { "?" };
    draw_text_world(
        &format!("{}={:.2f} slice", label, slice_coord),
        center_x - radius, center_y - radius - 0.04,
        camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
        0.025,
        Color::new(color.r, color.g, color.b, 0.5),
    );

    // Crosshairs
    draw_line_world(
        center_x - radius * 0.8, center_y, center_x + radius * 0.8, center_y,
        camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
        Color::new(color.r * 0.3, color.g * 0.3, color.b * 0.3, 0.15), 0.5,
    );
    draw_line_world(
        center_x, center_y - radius * 0.8, center_x, center_y + radius * 0.8,
        camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
        Color::new(color.r * 0.3, color.g * 0.3, color.b * 0.3, 0.15), 0.5,
    );
}

/// Draw a 4D rotation plane indicator.
/// Shows two axes that are being rotated into each other.
pub fn draw_rotation_plane(
    center_x: f32, center_y: f32, size: f32,
    axis1: &str, axis2: &str, angle: f32,
    camera_pos_x: f32, camera_pos_y: f32,
    zoom: f32, screen_w: f32, screen_h: f32,
    color: Color,
) {
    // Draw a tilted ellipse representing the rotation plane
    let segments = 32;
    for i in 0..segments {
        let a1 = std::f32::consts::TAU * i as f32 / segments as f32;
        let a2 = std::f32::consts::TAU * (i + 1) as f32 / segments as f32;

        let px1 = center_x + size * 0.5 * a1.cos() + size * 0.2 * a1.sin() * angle.cos();
        let py1 = center_y + size * 0.5 * a1.sin() + size * 0.2 * a1.sin() * angle.sin();
        let px2 = center_x + size * 0.5 * a2.cos() + size * 0.2 * a2.sin() * angle.cos();
        let py2 = center_y + size * 0.5 * a2.sin() + size * 0.2 * a2.sin() * angle.sin();

        draw_line_world(
            px1, py1, px2, py2,
            camera_pos_x, camera_pos_y, zoom, screen_w, screen_h,
            Color::new(color.r, color.g, color.b, 0.2), 0.8,
        );
    }

    // Axis labels
    draw_text_world(axis1, center_x + size * 0.55, center_y - 0.01,
        camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, 0.025,
        Color::new(color.r, color.g, color.b, 0.5));
    draw_text_world(axis2, center_x + size * 0.2 * angle.cos() - 0.01,
        center_y + size * 0.55 + size * 0.2 * angle.sin(),
        camera_pos_x, camera_pos_y, zoom, screen_w, screen_h, 0.025,
        Color::new(color.r, color.g, color.b, 0.5));
}