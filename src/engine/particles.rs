use macroquad::prelude::*;

/// Shared particle system for use across all levels.
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub size: f32,
    pub size_end: f32,
    pub gravity: f32,
    pub friction: f32,
}

impl Particle {
    pub fn new(x: f32, y: f32, color: Color) -> Self {
        let life = rand::gen_range(0.4_f32, 1.8);
        Self {
            x,
            y,
            vx: rand::gen_range(-0.6, 0.6),
            vy: rand::gen_range(-0.6, 0.6),
            life,
            max_life: life,
            color,
            size: rand::gen_range(0.004, 0.014),
            size_end: 0.0,
            gravity: 0.0,
            friction: 0.98,
        }
    }

    pub fn with_velocity(x: f32, y: f32, vx: f32, vy: f32, color: Color) -> Self {
        let life = rand::gen_range(0.3_f32, 1.2);
        Self {
            x, y, vx, vy,
            life,
            max_life: life,
            color,
            size: rand::gen_range(0.005, 0.015),
            size_end: 0.0,
            gravity: 0.0,
            friction: 0.98,
        }
    }

    pub fn burst(x: f32, y: f32, color: Color, count: usize) -> Vec<Particle> {
        (0..count).map(|_| {
            let angle = rand::gen_range(0.0_f32, std::f32::consts::TAU);
            let speed = rand::gen_range(0.3, 1.5);
            let life = rand::gen_range(0.3_f32, 1.0);
            Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life,
                max_life: life,
                color,
                size: rand::gen_range(0.005, 0.018),
                size_end: 0.0,
                gravity: 0.0,
                friction: 0.96,
            }
        }).collect()
    }

    pub fn update(&mut self, dt: f32) {
        self.vy += self.gravity * dt;
        self.vx *= self.friction;
        self.vy *= self.friction;
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.life -= dt;
    }

    pub fn alive(&self) -> bool {
        self.life > 0.0
    }

    pub fn alpha(&self) -> f32 {
        (self.life / self.max_life).clamp(0.0, 1.0)
    }

    pub fn current_size(&self) -> f32 {
        let t = 1.0 - self.alpha();
        self.size + (self.size_end - self.size) * t
    }
}

/// Manages a pool of particles.
pub struct ParticleSystem {
    particles: Vec<Particle>,
    max_particles: usize,
}

impl ParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self { particles: Vec::with_capacity(max_particles), max_particles }
    }

    pub fn emit(&mut self, p: Particle) {
        if self.particles.len() < self.max_particles {
            self.particles.push(p);
        }
    }

    pub fn emit_burst(&mut self, x: f32, y: f32, color: Color, count: usize) {
        let to_add = count.min(self.max_particles - self.particles.len());
        for p in Particle::burst(x, y, color, to_add) {
            self.particles.push(p);
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.update(dt);
        }
        self.particles.retain(|p| p.alive());
    }

    pub fn draw(
        &self,
        cam_x: f32, cam_y: f32,
        zoom: f32, sw: f32, sh: f32,
    ) {
        use crate::engine::renderer::draw_circle_world;
        for p in &self.particles {
            draw_circle_world(
                p.x, p.y, p.current_size(),
                cam_x, cam_y, zoom, sw, sh,
                Color::new(p.color.r, p.color.g, p.color.b, p.alpha()),
            );
        }
    }

    pub fn count(&self) -> usize {
        self.particles.len()
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }
}