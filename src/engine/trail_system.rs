use macroquad::prelude::*;

/// A point in the trail.
#[derive(Clone)]
struct TrailPoint {
    x: f32,
    y: f32,
    life: f32,
    max_life: f32,
}

/// Manages a fading trail behind the player.
pub struct TrailSystem {
    points: Vec<TrailPoint>,
    max_points: usize,
    decay_rate: f32,
    emit_distance: f32,  // minimum distance between trail points
    last_x: f32,
    last_y: f32,
    color: Color,
}

impl TrailSystem {
    pub fn new(color: Color, max_points: usize) -> Self {
        Self {
            points: Vec::with_capacity(max_points),
            max_points,
            decay_rate: 3.0,
            emit_distance: 0.008,
            last_x: f32::MAX,
            last_y: f32::MAX,
            color,
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_decay(&mut self, rate: f32) {
        self.decay_rate = rate;
    }

    /// Add a new trail point if the player has moved enough.
    pub fn update(&mut self, x: f32, y: f32, dt: f32) {
        // Check distance from last point
        let dx = x - self.last_x;
        let dy = y - self.last_y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > self.emit_distance {
            self.points.push(TrailPoint {
                x, y, life: 1.0, max_life: 1.0,
            });
            self.last_x = x;
            self.last_y = y;
        }

        // Decay
        for p in &mut self.points {
            p.life -= self.decay_rate * dt;
        }
        self.points.retain(|p| p.life > 0.0);

        // Cap
        if self.points.len() > self.max_points {
            self.points.drain(..self.points.len() - self.max_points);
        }
    }

    /// Emit a burst of trail points (e.g., on dash).
    pub fn emit_burst(&mut self, x: f32, y: f32, count: usize) {
        for _ in 0..count {
            self.points.push(TrailPoint {
                x: x + rand::gen_range(-0.01, 0.01),
                y: y + rand::gen_range(-0.01, 0.01),
                life: 1.0, max_life: 1.0,
            });
        }
    }

    /// Clear all trail points.
    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// Draw the trail.
    pub fn draw(&self, cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32) {
        use crate::engine::renderer::draw_circle_world;
        for p in &self.points {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            let size = 0.015 * alpha;
            draw_circle_world(p.x, p.y, size, cam_x, cam_y, zoom, sw, sh,
                Color::new(self.color.r, self.color.g, self.color.b, alpha * 0.5));
        }
    }
}