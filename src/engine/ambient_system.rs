use macroquad::prelude::*;

/// Dynamic ambient background effects that adapt to the current geometry type.
/// Creates visual atmosphere: floating particles, grid distortions, curvature haze.
pub struct AmbientSystem {
    /// Floating background particles for atmosphere
    particles: Vec<AmbientParticle>,
    /// Current geometry type for visual adaptation
    pub geometry_mode: GeometryMode,
    /// Animation time accumulator
    time: f32,
    /// Enable/disable
    pub enabled: bool,
    /// Background star field for higher-dimensional levels
    stars: Vec<(f32, f32, f32, f32)>, // x, y, brightness, twinkle_phase
}

#[derive(Clone, Copy, PartialEq)]
pub enum GeometryMode {
    Hyperbolic,
    Spherical,
    Euclidean,
    Hyperspace4D,
    Hyperspace5D,
    Hyperbolic4D,
    Hyperbolic5D,
    Blended { hyperbolic: f32, spherical: f32 },
}

impl GeometryMode {
    pub fn from_blend(hyperbolic: f32, spherical: f32) -> Self {
        GeometryMode::Blended { hyperbolic, spherical }
    }
}

struct AmbientParticle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: f32,
    alpha: f32,
    life: f32,
    max_life: f32,
    color: Color,
    /// For geometry-aware movement
    follows_geodesic: bool,
    angle: f32,
}

impl AmbientSystem {
    pub fn new() -> Self {
        let mut stars = Vec::new();
        for _ in 0..80 {
            stars.push((
                rand::gen_range(0.0_f32, 1.0),
                rand::gen_range(0.0_f32, 1.0),
                rand::gen_range(0.1_f32, 0.6),
                rand::gen_range(0.0_f32, std::f32::consts::TAU),
            ));
        }
        Self {
            particles: Vec::with_capacity(100),
            geometry_mode: GeometryMode::Euclidean,
            time: 0.0,
            enabled: true,
            stars,
        }
    }

    pub fn set_mode(&mut self, mode: GeometryMode) {
        self.geometry_mode = mode;
    }

    pub fn update(&mut self, dt: f32) {
        if !self.enabled { return; }
        self.time += dt;

        // Emit new ambient particles periodically
        let emit_rate = match self.geometry_mode {
            GeometryMode::Hyperbolic => 0.3,
            GeometryMode::Spherical => 0.2,
            GeometryMode::Hyperspace4D | GeometryMode::Hyperspace5D => 0.5,
            GeometryMode::Hyperbolic4D | GeometryMode::Hyperbolic5D => 0.4,
            _ => 0.15,
        };

        if rand::gen_range(0.0_f32, 1.0) < emit_rate * dt * 60.0 {
            self.emit_particle();
        }

        // Update existing particles
        for p in &mut self.particles {
            p.life -= dt;
            
            match self.geometry_mode {
                GeometryMode::Hyperbolic | GeometryMode::Hyperbolic4D | GeometryMode::Hyperbolic5D => {
                    // Particles spiral inward (converging toward center, like the Poincaré disk)
                    let dx = -p.x * 0.3;
                    let dy = -p.y * 0.3;
                    p.vx += dx * dt;
                    p.vy += dy * dt;
                    p.vx *= 0.99;
                    p.vy *= 0.99;
                }
                GeometryMode::Spherical => {
                    // Particles drift in great-circle-like paths
                    let r = (p.x * p.x + p.y * p.y).sqrt().max(0.01);
                    let tx = -p.y / r;
                    let ty = p.x / r;
                    p.vx += tx * dt * 0.5;
                    p.vy += ty * dt * 0.5;
                    p.vx *= 0.98;
                    p.vy *= 0.98;
                }
                GeometryMode::Hyperspace4D | GeometryMode::Hyperspace5D => {
                    // 4D/5D: particles have "extra dimension" oscillation
                    let extra = (self.time * 2.0 + p.angle).sin() * 0.3;
                    p.vx += extra * dt;
                    p.vy += (self.time * 1.7 + p.angle * 1.3).cos() * 0.2 * dt;
                    p.vx *= 0.97;
                    p.vy *= 0.97;
                }
                GeometryMode::Blended { hyperbolic, spherical } => {
                    // Blend between hyperbolic and spherical
                    let dx_h = -p.x * 0.3;
                    let dy_h = -p.y * 0.3;
                    let r = (p.x * p.x + p.y * p.y).sqrt().max(0.01);
                    let tx_s = -p.y / r * 0.5;
                    let ty_s = p.x / r * 0.5;
                    p.vx += (dx_h * hyperbolic + tx_s * spherical) * dt;
                    p.vy += (dy_h * hyperbolic + ty_s * spherical) * dt;
                    p.vx *= 0.985;
                    p.vy *= 0.985;
                }
                _ => {
                    // Euclidean: gentle drift
                    p.vx *= 0.995;
                    p.vy *= 0.995;
                }
            }

            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }

        self.particles.retain(|p| p.life > 0.0);
    }

    fn emit_particle(&mut self) {
        if self.particles.len() >= 100 { return; }

        let (x, y) = match self.geometry_mode {
            GeometryMode::Hyperbolic | GeometryMode::Hyperbolic4D | GeometryMode::Hyperbolic5D => {
                // Emit near the boundary for dramatic effect
                let angle = rand::gen_range(0.0_f32, std::f32::consts::TAU);
                let r = rand::gen_range(0.5_f32, 0.95);
                (r * angle.cos(), r * angle.sin())
            }
            GeometryMode::Spherical => {
                // Emit randomly across the map
                (rand::gen_range(-0.9_f32, 0.9), rand::gen_range(-0.9_f32, 0.9))
            }
            _ => {
                (rand::gen_range(-0.8_f32, 0.8), rand::gen_range(-0.8_f32, 0.8))
            }
        };

        let color = match self.geometry_mode {
            GeometryMode::Hyperbolic | GeometryMode::Hyperbolic4D => 
                Color::new(0.0, rand::gen_range(0.4_f32, 0.8), rand::gen_range(0.6_f32, 1.0), 1.0),
            GeometryMode::Spherical => 
                Color::new(rand::gen_range(0.6_f32, 1.0), rand::gen_range(0.3_f32, 0.6), 0.0, 1.0),
            GeometryMode::Hyperspace4D => 
                Color::new(rand::gen_range(0.2_f32, 0.5), 0.0, rand::gen_range(0.7_f32, 1.0), 1.0),
            GeometryMode::Hyperspace5D => 
                Color::new(1.0, rand::gen_range(0.2_f32, 0.5), 0.0, 1.0),
            GeometryMode::Hyperbolic5D => 
                Color::new(0.0, rand::gen_range(0.6_f32, 1.0), rand::gen_range(0.2_f32, 0.5), 1.0),
            _ => 
                Color::new(0.3, 0.3, 0.4, 1.0),
        };

        self.particles.push(AmbientParticle {
            x, y,
            vx: rand::gen_range(-0.1_f32, 0.1),
            vy: rand::gen_range(-0.1_f32, 0.1),
            size: rand::gen_range(0.002_f32, 0.006),
            alpha: rand::gen_range(0.2_f32, 0.5),
            life: rand::gen_range(2.0_f32, 6.0),
            max_life: 6.0,
            color,
            follows_geodesic: rand::gen_range(0.0_f32, 1.0) > 0.5,
            angle: rand::gen_range(0.0_f32, std::f32::consts::TAU),
        });
    }

    /// Draw the ambient background layer.
    /// Call BEFORE drawing world objects.
    pub fn draw_background(&self, sw: f32, sh: f32) {
        if !self.enabled { return; }

        // Background gradient based on geometry
        let bg = match self.geometry_mode {
            GeometryMode::Hyperbolic | GeometryMode::Hyperbolic4D | GeometryMode::Hyperbolic5D => {
                let pulse = (self.time * 0.3).sin() * 0.02;
                Color::new(0.02 + pulse, 0.02, 0.06 + pulse, 1.0)
            }
            GeometryMode::Spherical => {
                let pulse = (self.time * 0.2).sin() * 0.015;
                Color::new(0.06 + pulse, 0.03, 0.02, 1.0)
            }
            GeometryMode::Hyperspace4D => {
                let pulse = (self.time * 0.4).sin() * 0.02;
                Color::new(0.04, 0.02, 0.08 + pulse, 1.0)
            }
            GeometryMode::Hyperspace5D => {
                let pulse = (self.time * 0.35).sin() * 0.02;
                Color::new(0.06 + pulse, 0.02, 0.04, 1.0)
            }
            GeometryMode::Blended { hyperbolic, spherical } => {
                let r = spherical * 0.06;
                let b = hyperbolic * 0.06;
                Color::new(r, 0.02, b, 1.0)
            }
            _ => Color::new(0.02, 0.02, 0.04, 1.0),
        };
        clear_background(bg);

        // Stars for higher-dimensional levels
        match self.geometry_mode {
            GeometryMode::Hyperspace4D | GeometryMode::Hyperspace5D |
            GeometryMode::Hyperbolic4D | GeometryMode::Hyperbolic5D => {
                for &(sx, sy, brightness, phase) in &self.stars {
                    let twinkle = (self.time * 2.0 + phase).sin() * 0.3 + 0.7;
                    let alpha = brightness * twinkle;
                    draw_circle(sx * sw, sy * sh, 1.5, Color::new(0.7, 0.7, 1.0, alpha));
                }
            }
            _ => {}
        }
    }

    /// Draw ambient particles in world space.
    pub fn draw_world(&self, cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32) {
        if !self.enabled { return; }
        use crate::engine::renderer::draw_circle_world;
        for p in &self.particles {
            let life_frac = (p.life / p.max_life).clamp(0.0, 1.0);
            let fade = if life_frac > 0.8 { (1.0 - life_frac) / 0.2 } else { life_frac.min(1.0) };
            let a = p.alpha * fade;
            
            draw_circle_world(
                p.x, p.y, p.size,
                cam_x, cam_y, zoom, sw, sh,
                Color::new(p.color.r, p.color.g, p.color.b, a * 0.4),
            );
        }
    }

    /// Draw geometry-specific overlay effects (curvature haze, dimension ripples).
    pub fn draw_overlay(&self, sw: f32, sh: f32, player_x: f32, player_y: f32) {
        if !self.enabled { return; }

        match self.geometry_mode {
            GeometryMode::Hyperbolic => {
                // Subtle converging radial lines near boundary
                let cx = sw / 2.0 - player_x * sw * 0.45;
                let cy = sh / 2.0 - player_y * sh * 0.45;
                for i in 0..12 {
                    let angle = std::f32::consts::TAU * i as f32 / 12.0 + self.time * 0.05;
                    let inner_r = 80.0;
                    let outer_r = 200.0 + (self.time * 10.0 + i as f32 * 50.0).sin() * 30.0;
                    draw_line(
                        cx + inner_r * angle.cos(), cy + inner_r * angle.sin(),
                        cx + outer_r * angle.cos(), cy + outer_r * angle.sin(),
                        1.0, Color::new(0.0, 0.3, 0.5, 0.08),
                    );
                }
            }
            GeometryMode::Hyperspace4D | GeometryMode::Hyperspace5D => {
                // Dimension ripple effect
                let dims = if self.geometry_mode == GeometryMode::Hyperspace4D { 4 } else { 5 };
                for d in 0..dims {
                    let phase = self.time * 0.8 + d as f32 * 1.2;
                    let r = 50.0 + (phase).sin() * 30.0 + d as f32 * 40.0;
                    let alpha = 0.03 + (phase * 0.5).sin() * 0.02;
                    draw_circle_lines(sw / 2.0, sh / 2.0, r, 1.0,
                        Color::new(0.4, 0.2, 0.8, alpha));
                }
            }
            _ => {}
        }
    }
}