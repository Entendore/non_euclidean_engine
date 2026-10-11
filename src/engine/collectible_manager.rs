use macroquad::prelude::*;
use crate::geometry::GeoPoint;
use crate::engine::game::{Collectible, LevelRuntime};
use crate::engine::renderer;

/// Centralized collectible management system.
/// Handles updating, collection checking, rendering, and proximity indicators.
pub struct CollectibleManager {
    pub collectibles: Vec<Collectible>,
    pub collected_count: usize,
    pub base_score: u32,
}

impl CollectibleManager {
    pub fn new(base_score: u32) -> Self {
        Self { collectibles: Vec::new(), collected_count: 0, base_score }
    }

    pub fn add(&mut self, c: Collectible) {
        self.collectibles.push(c);
    }

    pub fn total(&self) -> usize {
        self.collectibles.len()
    }

    pub fn fraction(&self) -> f32 {
        if self.collectibles.is_empty() { return 1.0; }
        self.collected_count as f32 / self.collectibles.len() as f32
    }

    pub fn all_collected(&self) -> bool {
        self.collected_count >= self.collectibles.len()
    }

    /// Update animations. Call every frame.
    pub fn update(&mut self, dt: f32) {
        for c in &mut self.collectibles { c.update(dt); }
    }

    /// Check collection with a given distance function.
    /// Returns the score earned this frame (0 if nothing collected).
    pub fn check_collection<F>(&mut self, player: GeoPoint, dist_fn: F, runtime: &mut LevelRuntime) -> u32
    where F: Fn(GeoPoint, GeoPoint) -> f32
    {
        let collect_dist = 0.035;
        let mut earned = 0u32;

        for c in &mut self.collectibles {
            if !c.collected {
                let dist = dist_fn(player, c.pos);
                if dist < collect_dist {
                    c.collected = true;
                    self.collected_count += 1;
                    let score = runtime.combo.on_collect(runtime.level_time, self.base_score);
                    earned += score;
                    runtime.particles.emit_burst(c.pos.x, c.pos.y, c.color, 15);
                    runtime.effects.shake(3.0, 0.15);
                    runtime.effects.flash(Color::new(1.0, 1.0, 0.3, 1.0), 0.1);
                }
            }
        }
        earned
    }

    /// Draw all uncollected collectibles with standard rendering.
    pub fn draw(&self, cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32) {
        for c in &self.collectibles {
            if c.collected { continue; }
            let er = c.effective_radius();
            let by = c.bob_y();

            // Glow
            renderer::draw_circle_world(c.pos.x, c.pos.y + by, er * 2.0,
                cam_x, cam_y, zoom, sw, sh,
                Color::new(c.color.r, c.color.g, c.color.b, 0.2));
            // Body
            renderer::draw_circle_world(c.pos.x, c.pos.y + by, er,
                cam_x, cam_y, zoom, sw, sh, c.color);
            // Label
            renderer::draw_text_world(&c.label.to_string(),
                c.pos.x - 0.01, c.pos.y + by - 0.01,
                cam_x, cam_y, zoom, sw, sh, 0.04, WHITE);
        }
    }

    /// Draw uncollected collectibles with conformal scaling (hyperbolic).
    pub fn draw_conformal(&self, cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32) {
        for c in &self.collectibles {
            if c.collected { continue; }
            let er = c.effective_radius();
            let by = c.bob_y();

            renderer::draw_circle_world_conformal(c.pos.x, c.pos.y + by, er * 2.0,
                cam_x, cam_y, zoom, sw, sh,
                Color::new(c.color.r, c.color.g, c.color.b, 0.2));
            renderer::draw_circle_world_conformal(c.pos.x, c.pos.y + by, er,
                cam_x, cam_y, zoom, sw, sh, c.color);
            renderer::draw_text_world_conformal(&c.label.to_string(),
                c.pos.x - 0.01, c.pos.y + by - 0.01,
                cam_x, cam_y, zoom, sw, sh, 0.04, WHITE);
        }
    }

    /// Draw uncollected collectibles with spherical scaling.
    pub fn draw_spherical(&self, cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32) {
        for c in &self.collectibles {
            if c.collected { continue; }
            let er = c.effective_radius();
            let by = c.bob_y();

            renderer::draw_circle_world_spherical(c.pos.x, c.pos.y + by, er * 2.0,
                cam_x, cam_y, zoom, sw, sh,
                Color::new(c.color.r, c.color.g, c.color.b, 0.2));
            renderer::draw_circle_world_spherical(c.pos.x, c.pos.y + by, er,
                cam_x, cam_y, zoom, sw, sh, c.color);
            renderer::draw_text_world(&c.label.to_string(),
                c.pos.x - 0.01, c.pos.y + by - 0.01,
                cam_x, cam_y, zoom, sw, sh, 0.04, WHITE);
        }
    }

    /// Draw proximity indicators for uncollected items.
    pub fn draw_proximity(&self, cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32) {
        let half_w = sw * 0.45 * zoom;
        let half_h = sh * 0.45 * zoom;
        for c in &self.collectibles {
            if c.collected { continue; }
            let sx = sw / 2.0 + (c.pos.x - cam_x) * half_w;
            let sy = sh / 2.0 + (c.pos.y - cam_y) * half_h;
            renderer::draw_proximity_indicator(sw / 2.0, sh / 2.0, sx, sy, sw, sh, c.color, &c.label.to_string());
        }
    }

    /// Get data for minimap display.
    pub fn minimap_data(&self) -> Vec<(f32, f32, bool, char)> {
        self.collectibles.iter().map(|c| (c.pos.x, c.pos.y, c.collected, c.label)).collect()
    }
}