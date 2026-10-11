use macroquad::prelude::*;
use crate::engine::game::Wall;

/// Procedural wall generation engine primitive.
/// Provides builders for common wall patterns used across levels.
pub struct WallBuilder;

impl WallBuilder {
    /// Generate a ring of wall segments at a given radius with configurable gaps.
    /// `gap_angles`: list of angles (radians) where gaps are centered.
    /// `gap_half_width`: half-width of each gap in radians.
    pub fn ring(center_x: f32, center_y: f32, radius: f32, segments: usize,
                gap_angles: &[f32], gap_half_width: f32,
                color: Color) -> Vec<Wall> {
        let mut walls = Vec::new();
        for i in 0..segments {
            let angle = std::f32::consts::TAU * i as f32 / segments as f32;
            let next_angle = std::f32::consts::TAU * (i + 1) as f32 / segments as f32;
            let gap_center = (angle + next_angle) / 2.0;

            let mut in_gap = false;
            for &ga in gap_angles {
                let diff = (gap_center - ga).abs();
                let diff = diff.min(std::f32::consts::TAU - diff);
                if diff < gap_half_width { in_gap = true; break; }
            }
            if in_gap { continue; }

            walls.push(Wall::new(
                center_x + radius * angle.cos(), center_y + radius * angle.sin(),
                center_x + radius * next_angle.cos(), center_y + radius * next_angle.sin(),
                color));
        }
        walls
    }

    /// Generate radial walls from inner_radius to outer_radius at given angles.
    pub fn radial_spokes(center_x: f32, center_y: f32,
                         inner_r: f32, outer_r: f32,
                         spoke_count: usize, angle_offset: f32,
                         color: Color) -> Vec<Wall> {
        let mut walls = Vec::new();
        for i in 0..spoke_count {
            let angle = std::f32::consts::TAU * i as f32 / spoke_count as f32 + angle_offset;
            walls.push(Wall::new(
                center_x + inner_r * angle.cos(), center_y + inner_r * angle.sin(),
                center_x + outer_r * angle.cos(), center_y + outer_r * angle.sin(),
                color));
        }
        walls
    }

    /// Generate a rectangular boundary (4 walls).
    pub fn rect(x: f32, y: f32, w: f32, h: f32, color: Color) -> Vec<Wall> {
        vec![
            Wall::new(x, y, x + w, y, color),
            Wall::new(x + w, y, x + w, y + h, color),
            Wall::new(x + w, y + h, x, y + h, color),
            Wall::new(x, y + h, x, y, color),
        ]
    }

    /// Generate horizontal wall band at a given y, with gaps.
    pub fn h_band(y: f32, x_start: f32, x_end: f32, segment_len: f32,
                  gap_centers: &[f32], gap_half: f32, color: Color) -> Vec<Wall> {
        let mut walls = Vec::new();
        let mut x = x_start;
        while x < x_end {
            let seg_end = (x + segment_len).min(x_end);
            let mid = (x + seg_end) / 2.0;
            let mut skip = false;
            for &gc in gap_centers {
                if (mid - gc).abs() < gap_half { skip = true; break; }
            }
            if !skip && (x - x_start).abs() > 0.001 {
                walls.push(Wall::new(x, y, seg_end, y, color));
            }
            x = seg_end;
        }
        walls
    }

    /// Generate vertical wall band at a given x, with gaps.
    pub fn v_band(x: f32, y_start: f32, y_end: f32, segment_len: f32,
                  gap_centers: &[f32], gap_half: f32, color: Color) -> Vec<Wall> {
        let mut walls = Vec::new();
        let mut y = y_start;
        while y < y_end {
            let seg_end = (y + segment_len).min(y_end);
            let mid = (y + seg_end) / 2.0;
            let mut skip = false;
            for &gc in gap_centers {
                if (mid - gc).abs() < gap_half { skip = true; break; }
            }
            if !skip && (y - y_start).abs() > 0.001 {
                walls.push(Wall::new(x, y, x, seg_end, color));
            }
            y = seg_end;
        }
        walls
    }

    /// Generate a grid of walls within a region.
    pub fn grid(x: f32, y: f32, w: f32, h: f32,
                cell_w: f32, cell_h: f32,
                wall_gaps: &[(f32, f32, f32, f32)], // (gx, gy, gw, gh) holes
                color: Color) -> Vec<Wall> {
        let mut walls = Vec::new();
        let h_lines = (h / cell_h).ceil() as i32;
        let v_lines = (w / cell_w).ceil() as i32;

        for i in 0..=h_lines {
            let ly = y + i as f32 * cell_h;
            if ly > y + h { break; }
            for j in 0..v_lines {
                let lx = x + j as f32 * cell_w;
                let seg_end = (lx + cell_w).min(x + w);
                let mid_x = (lx + seg_end) / 2.0;
                let mid_y = ly;
                let mut skip = false;
                for &(gx, gy, gw, gh) in wall_gaps {
                    if mid_x >= gx && mid_x <= gx + gw && mid_y >= gy && mid_y <= gy + gh {
                        skip = true; break;
                    }
                }
                if !skip {
                    walls.push(Wall::new(lx, ly, seg_end, ly, color));
                }
            }
        }
        for j in 0..=v_lines {
            let lx = x + j as f32 * cell_w;
            if lx > x + w { break; }
            for i in 0..h_lines {
                let ly = y + i as f32 * cell_h;
                let seg_end = (ly + cell_h).min(y + h);
                let mid_x = lx;
                let mid_y = (ly + seg_end) / 2.0;
                let mut skip = false;
                for &(gx, gy, gw, gh) in wall_gaps {
                    if mid_x >= gx && mid_x <= gx + gw && mid_y >= gy && mid_y <= gy + gh {
                        skip = true; break;
                    }
                }
                if !skip {
                    walls.push(Wall::new(lx, ly, lx, seg_end, color));
                }
            }
        }
        walls
    }

    /// Generate concentric rings with gaps at different angles per ring.
    pub fn concentric_rings(center_x: f32, center_y: f32,
                            radii: &[f32], segments_per_ring: usize,
                            gaps_per_ring: usize, color: Color) -> Vec<Wall> {
        let mut walls = Vec::new();
        for (ri, &radius) in radii.iter().enumerate() {
            // Rotate gap positions for each ring
            let gap_offset = std::f32::consts::FRAC_PI_2 / (ri as f32 + 1.0);
            let gap_angles: Vec<f32> = (0..gaps_per_ring)
                .map(|g| std::f32::consts::TAU * g as f32 / gaps_per_ring as f32 + gap_offset)
                .collect();
            walls.extend(Self::ring(center_x, center_y, radius, segments_per_ring,
                                     &gap_angles, 0.35, color));
        }
        walls
    }

    /// Generate a maze-like pattern using recursive-ish concentric rings + radial spokes.
    /// Returns a set of walls that form a hyperbolic-style maze.
    pub fn hyperbolic_maze(center_x: f32, center_y: f32,
                           max_radius: f32, ring_count: usize,
                           spoke_count: usize, color: Color) -> Vec<Wall> {
        let mut walls = Vec::new();
        let segment_count = ring_count * 6;

        for ring_i in 0..ring_count {
            let r = (ring_i + 1) as f32 / ring_count as f32 * max_radius;
            if r >= 0.98 { break; }

            // Alternate gap pattern for each ring
            let gap_positions = if ring_i % 2 == 0 { 3 } else { 4 };
            let gap_offset = if ring_i % 2 == 0 { 0.0 } else { std::f32::consts::FRAC_PI_4 };

            for seg_i in 0..segment_count {
                let angle = std::f32::consts::TAU * seg_i as f32 / segment_count as f32;
                let next_angle = std::f32::consts::TAU * (seg_i + 1) as f32 / segment_count as f32;
                let mid = (angle + next_angle) / 2.0;

                let mut in_gap = false;
                for g in 0..gap_positions {
                    let ga = std::f32::consts::TAU * g as f32 / gap_positions as f32 + gap_offset;
                    let diff = (mid - ga).abs().min(std::f32::consts::TAU - (mid - ga).abs());
                    if diff < 0.25 { in_gap = true; break; }
                }
                if in_gap { continue; }

                let x1 = center_x + r * angle.cos();
                let y1 = center_y + r * angle.sin();
                let x2 = center_x + r * next_angle.cos();
                let y2 = center_y + r * next_angle.sin();
                walls.push(Wall::new(x1, y1, x2, y2, color));
            }
        }

        // Radial spokes with gaps
        for spoke_i in 0..spoke_count {
            let base_angle = std::f32::consts::TAU * spoke_i as f32 / spoke_count as f32;
            // Create segments with gaps at ring boundaries
            for seg in 0..ring_count - 1 {
                let r1 = (seg + 1) as f32 / ring_count as f32 * max_radius * 0.85;
                let r2 = (seg + 2) as f32 / ring_count as f32 * max_radius * 0.95;
                if r2 >= 0.97 { continue; }
                // Skip some segments for passages
                if (spoke_i + seg) % 3 == 0 { continue; }

                walls.push(Wall::new(
                    center_x + r1 * base_angle.cos(), center_y + r1 * base_angle.sin(),
                    center_x + r2 * base_angle.cos(), center_y + r2 * base_angle.sin(),
                    Color::new(color.r * 0.8, color.g * 0.8, color.b * 0.8, color.a),
                ));
            }
        }
        walls
    }
}