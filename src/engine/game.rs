use macroquad::prelude::*;
use crate::geometry::{GeoPoint, GeometryType};
use crate::engine::particles::ParticleSystem;
use crate::engine::effects::ScreenEffects;

/// Global game state that persists across levels.
pub struct GameState {
    pub current_level: usize,
    pub total_score: u32,
    pub best_scores: [u32; 5],
    pub levels_completed: [bool; 5],
    pub show_help: bool,
    pub time_elapsed: f64,
    pub message: String,
    pub message_timer: f64,
    pub paused: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            current_level: 0,
            total_score: 0,
            best_scores: [0; 5],
            levels_completed: [false; 5],
            show_help: false,
            time_elapsed: 0.0,
            message: String::new(),
            message_timer: 0.0,
            paused: false,
        }
    }

    pub fn show_message(&mut self, msg: &str, duration: f64) {
        self.message = msg.to_string();
        self.message_timer = duration;
    }

    pub fn update(&mut self, dt: f64) {
        if self.paused { return; }
        self.time_elapsed += dt;
        if self.message_timer > 0.0 {
            self.message_timer -= dt;
            if self.message_timer <= 0.0 { self.message.clear(); }
        }
        if is_key_pressed(KeyCode::H) {
            self.show_help = !self.show_help;
        }
    }

    pub fn complete_level(&mut self, level_idx: usize, score: u32) {
        if level_idx > 0 && level_idx <= 5 {
            self.levels_completed[level_idx - 1] = true;
            self.total_score += score;
            if score > self.best_scores[level_idx - 1] {
                self.best_scores[level_idx - 1] = score;
            }
        }
    }
}

/// A collectible item in a level.
pub struct Collectible {
    pub pos: GeoPoint,
    pub collected: bool,
    pub radius: f32,
    pub color: Color,
    pub pulse_phase: f32,
    pub label: char,
    pub bob_offset: f32,
}

impl Collectible {
    pub fn new(x: f32, y: f32, label: char, color: Color) -> Self {
        Self {
            pos: GeoPoint::new(x, y),
            collected: false,
            radius: 0.03,
            color,
            pulse_phase: rand::gen_range(0.0_f32, std::f32::consts::TAU),
            label,
            bob_offset: rand::gen_range(0.0_f32, std::f32::consts::TAU),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pulse_phase += dt * 3.0;
        self.bob_offset += dt * 1.5;
    }

    pub fn effective_radius(&self) -> f32 {
        self.radius * (1.0 + 0.3 * self.pulse_phase.sin())
    }

    pub fn bob_y(&self) -> f32 {
        (self.bob_offset * 0.3).sin() * 0.008
    }
}

/// A wall/obstacle in a level.
pub struct Wall {
    pub p1: GeoPoint,
    pub p2: GeoPoint,
    pub color: Color,
    pub thickness: f32,
}

impl Wall {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32, color: Color) -> Self {
        Self {
            p1: GeoPoint::new(x1, y1),
            p2: GeoPoint::new(x2, y2),
            color,
            thickness: 0.008,
        }
    }
}

/// Different portal types that create different spatial illusions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortalType {
    /// Standard bidirectional portal with spatial warp illusion.
    Normal,
    /// One-way portal — can only enter from A, exit at B.
    OneWayAB,
    /// One-way portal — can only enter from B, exit at A.
    OneWayBA,
    /// Mirror portal — flips the player's X coordinate on teleport.
    Mirror,
    /// Geometry-shift portal — teleports between different curvature spaces.
    GeometryShift,
    /// Auto-teleport portal — no action key needed, walk through it.
    AutoWarp,
    /// Recursive portal — portal within a portal (visual + teleport chain).
    Recursive,
    /// Dimensional portal — shifts between 4D and 5D spaces.
    DimensionalShift,
}

impl PortalType {
    pub fn label(&self) -> &'static str {
        match self {
            PortalType::Normal => "N",
            PortalType::OneWayAB => "A>B",
            PortalType::OneWayBA => "B>A",
            PortalType::Mirror => "M",
            PortalType::GeometryShift => "G",
            PortalType::AutoWarp => "W",
            PortalType::Recursive => "R",
            PortalType::DimensionalShift => "D",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PortalType::Normal => "Bidirectional portal",
            PortalType::OneWayAB => "One-way: A to B only",
            PortalType::OneWayBA => "One-way: B to A only",
            PortalType::Mirror => "Mirror portal (flips X)",
            PortalType::GeometryShift => "Geometry shift portal",
            PortalType::AutoWarp => "Auto-warp (walk through)",
            PortalType::Recursive => "Recursive portal",
            PortalType::DimensionalShift => "Dimensional shift (4D/5D)",
        }
    }

    /// Whether this portal teleports on proximity (no action key).
    pub fn is_auto(&self) -> bool {
        matches!(self, PortalType::AutoWarp | PortalType::Recursive | PortalType::DimensionalShift)
    }
}

/// A portal that teleports the player between two locations.
/// Enhanced with portal types, illusion rendering, and spatial warp effects.
pub struct Portal {
    pub pos_a: GeoPoint,
    pub pos_b: GeoPoint,
    pub radius: f32,
    pub cooldown: f32,
    pub color_a: Color,
    pub color_b: Color,
    pub angle: f32,
    pub portal_type: PortalType,
    pub show_window: bool,
    pub show_warp: bool,
    pub mirror_flip_x: bool,
    pub label: String,
    /// For recursive portals: chain to another portal index.
    pub chain_to: Option<usize>,
    /// Whether to draw the geodesic link between A and B.
    pub show_geodesic_link: bool,
    /// Whether to draw the wormhole corridor.
    pub show_corridor: bool,
}

impl Portal {
    pub fn new(ax: f32, ay: f32, bx: f32, by: f32) -> Self {
        Self {
            pos_a: GeoPoint::new(ax, ay),
            pos_b: GeoPoint::new(bx, by),
            radius: 0.04,
            cooldown: 0.0,
            color_a: PURPLE,
            color_b: Color::new(0.9, 0.0, 0.9, 1.0),
            angle: 0.0,
            portal_type: PortalType::Normal,
            show_window: true,
            show_warp: true,
            mirror_flip_x: false,
            label: String::new(),
            chain_to: None,
            show_geodesic_link: true,
            show_corridor: false,
        }
    }

    pub fn with_type(ax: f32, ay: f32, bx: f32, by: f32, portal_type: PortalType) -> Self {
        let mut p = Self::new(ax, ay, bx, by);
        p.portal_type = portal_type;
        match portal_type {
            PortalType::Mirror => {
                p.mirror_flip_x = true;
                p.color_a = Color::new(0.2, 0.8, 1.0, 1.0);
                p.color_b = Color::new(1.0, 0.8, 0.2, 1.0);
                p.label = "M".to_string();
            }
            PortalType::GeometryShift => {
                p.color_a = Color::new(0.0, 1.0, 0.5, 1.0);
                p.color_b = Color::new(1.0, 0.3, 0.0, 1.0);
                p.show_warp = true;
                p.label = "G".to_string();
            }
            PortalType::OneWayAB => {
                p.color_b = Color::new(0.5, 0.5, 0.5, 1.0);
                p.label = "A>B".to_string();
            }
            PortalType::OneWayBA => {
                p.color_a = Color::new(0.5, 0.5, 0.5, 1.0);
                p.label = "B>A".to_string();
            }
            PortalType::AutoWarp => {
                p.color_a = Color::new(0.0, 0.7, 1.0, 1.0);
                p.color_b = Color::new(1.0, 0.5, 0.0, 1.0);
                p.label = "W".to_string();
                p.show_geodesic_link = true;
                p.show_corridor = true;
            }
            PortalType::Recursive => {
                p.color_a = Color::new(0.8, 0.0, 1.0, 1.0);
                p.color_b = Color::new(0.0, 1.0, 0.8, 1.0);
                p.label = "R".to_string();
                p.show_geodesic_link = true;
            }
            PortalType::DimensionalShift => {
                p.color_a = Color::new(1.0, 0.3, 0.0, 1.0);
                p.color_b = Color::new(0.0, 0.3, 1.0, 1.0);
                p.label = "D".to_string();
                p.show_geodesic_link = true;
                p.show_corridor = true;
                p.show_warp = true;
            }
            PortalType::Normal => {}
        }
        p
    }

    pub fn can_enter_from_a(&self) -> bool {
        match self.portal_type {
            PortalType::Normal | PortalType::OneWayAB | PortalType::Mirror |
            PortalType::GeometryShift | PortalType::AutoWarp | PortalType::Recursive |
            PortalType::DimensionalShift => true,
            PortalType::OneWayBA => false,
        }
    }

    pub fn can_enter_from_b(&self) -> bool {
        match self.portal_type {
            PortalType::Normal | PortalType::OneWayBA | PortalType::Mirror |
            PortalType::GeometryShift | PortalType::AutoWarp | PortalType::Recursive |
            PortalType::DimensionalShift => true,
            PortalType::OneWayAB => false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.angle += dt * 2.0;
        if self.cooldown > 0.0 { self.cooldown -= dt; }
    }
}

// ==================== CURVATURE ZONES ====================

/// A zone with a specific curvature value that affects rendering.
pub struct CurvatureZone {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    /// -1.0 = hyperbolic, 0.0 = euclidean, 1.0 = spherical
    pub curvature: f32,
    pub color: Color,
    pub label: String,
}

impl CurvatureZone {
    pub fn new(x: f32, y: f32, radius: f32, curvature: f32, color: Color, label: &str) -> Self {
        Self { x, y, radius, curvature, color, label: label.to_string() }
    }

    pub fn influence_at(&self, px: f32, py: f32) -> f32 {
        let dx = px - self.x;
        let dy = py - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist >= self.radius { return 0.0; }
        1.0 - dist / self.radius
    }
}

// ==================== SIZE ILLUSION ====================

pub struct SizeIllusionPair {
    pub pos_a: GeoPoint,
    pub pos_b: GeoPoint,
    pub radius: f32,
    pub color: Color,
    pub label: String,
}

impl SizeIllusionPair {
    pub fn new(ax: f32, ay: f32, bx: f32, by: f32, radius: f32, color: Color, label: &str) -> Self {
        Self {
            pos_a: GeoPoint::new(ax, ay),
            pos_b: GeoPoint::new(bx, by),
            radius,
            color,
            label: label.to_string(),
        }
    }
}

// ==================== ENEMY ====================

pub struct Enemy {
    pub pos: GeoPoint,
    pub p1: GeoPoint,
    pub p2: GeoPoint,
    pub speed: f32,
    pub t: f32,
    pub direction: f32,
    pub radius: f32,
    pub color: Color,
    pub phase: f32,
    pub alert_radius: f32,
}

impl Enemy {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32, speed: f32) -> Self {
        Self {
            pos: GeoPoint::new(x1, y1),
            p1: GeoPoint::new(x1, y1),
            p2: GeoPoint::new(x2, y2),
            speed,
            t: 0.0,
            direction: 1.0,
            radius: 0.025,
            color: RED,
            phase: rand::gen_range(0.0_f32, std::f32::consts::TAU),
            alert_radius: 0.08,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.phase += dt * 4.0;
        let dist = self.p1.distance_to(self.p2);
        if dist > 0.001 {
            self.t += self.direction * self.speed * dt / dist;
            if self.t >= 1.0 { self.t = 1.0; self.direction = -1.0; }
            if self.t <= 0.0 { self.t = 0.0; self.direction = 1.0; }
        }
        self.pos = self.p1.lerp(self.p2, self.t);
    }

    pub fn distance_to_player(&self, player: GeoPoint) -> f32 {
        self.pos.distance_to(player)
    }
}

// ==================== DASH ====================

pub struct DashState {
    pub energy: f32,
    pub max_energy: f32,
    pub recharge_rate: f32,
    pub dash_speed_mult: f32,
    pub dash_duration: f32,
    pub is_dashing: bool,
    pub dash_timer: f32,
    pub dash_cooldown: f32,
    pub dash_cooldown_timer: f32,
    pub trail: Vec<(f32, f32, f32)>,
}

impl DashState {
    pub fn new() -> Self {
        Self {
            energy: 1.0, max_energy: 1.0, recharge_rate: 0.25,
            dash_speed_mult: 3.5, dash_duration: 0.15,
            is_dashing: false, dash_timer: 0.0,
            dash_cooldown: 0.0, dash_cooldown_timer: 0.0,
            trail: Vec::new(),
        }
    }

    pub fn try_dash(&mut self) -> bool {
        if self.energy >= 0.3 && !self.is_dashing && self.dash_cooldown_timer <= 0.0 {
            self.is_dashing = true;
            self.dash_timer = self.dash_duration;
            self.energy -= 0.3;
            self.dash_cooldown_timer = self.dash_cooldown;
            return true;
        }
        false
    }

    pub fn update(&mut self, dt: f32, player_x: f32, player_y: f32) {
        if !self.is_dashing { self.energy = (self.energy + self.recharge_rate * dt).min(self.max_energy); }
        if self.is_dashing {
            self.dash_timer -= dt;
            self.trail.push((player_x, player_y, 1.0));
            if self.dash_timer <= 0.0 {
                self.is_dashing = false;
                self.dash_cooldown = 0.3;
            }
        }
        if self.dash_cooldown_timer > 0.0 { self.dash_cooldown_timer -= dt; }
        for trail in &mut self.trail { trail.2 -= dt * 3.0; }
        self.trail.retain(|(_, _, a)| *a > 0.0);
        if self.trail.len() > 40 { self.trail.drain(..self.trail.len() - 40); }
    }

    pub fn speed_multiplier(&self) -> f32 {
        if self.is_dashing { self.dash_speed_mult } else { 1.0 }
    }
}

// ==================== COMBO ====================

pub struct ComboSystem {
    pub combo: u32,
    pub combo_timer: f32,
    pub combo_window: f32,
    pub max_multiplier: u32,
    pub last_collect_time: f32,
    pub total_bonus: u32,
    pub display_text: String,
    pub display_timer: f32,
}

impl ComboSystem {
    pub fn new() -> Self {
        Self {
            combo: 0, combo_timer: 0.0, combo_window: 3.0, max_multiplier: 5,
            last_collect_time: 0.0, total_bonus: 0,
            display_text: String::new(), display_timer: 0.0,
        }
    }

    pub fn on_collect(&mut self, time: f32, base_score: u32) -> u32 {
        if time - self.last_collect_time < self.combo_window {
            self.combo += 1;
        } else {
            self.combo = 1;
        }
        self.last_collect_time = time;
        self.combo_timer = self.combo_window;
        let mult = self.combo.min(self.max_multiplier) as u32;
        let score = base_score * mult;
        self.total_bonus += score - base_score;
        if mult > 1 {
            self.display_text = format!("x{} COMBO! +{}", mult, score);
        } else {
            self.display_text = format!("+{}", score);
        }
        self.display_timer = 1.5;
        score
    }

    pub fn update(&mut self, dt: f32) {
        if self.combo_timer > 0.0 {
            self.combo_timer -= dt;
            if self.combo_timer <= 0.0 { self.combo = 0; }
        }
        if self.display_timer > 0.0 { self.display_timer -= dt; }
    }

    pub fn multiplier(&self) -> u32 {
        if self.combo_timer > 0.0 { self.combo.min(self.max_multiplier) } else { 0 }
    }
}

// ==================== LEVEL RESULT & RUNTIME ====================

pub enum LevelResult {
    Playing,
    Completed(u32),
    Failed(String),
}

/// Shared per-level runtime state.
pub struct LevelRuntime {
    pub particles: ParticleSystem,
    pub effects: ScreenEffects,
    pub dash: DashState,
    pub combo: ComboSystem,
    pub enemies: Vec<Enemy>,
    pub invuln_timer: f32,
    pub level_time: f32,
}

impl LevelRuntime {
    pub fn new() -> Self {
        Self {
            particles: ParticleSystem::new(500),
            effects: ScreenEffects::new(),
            dash: DashState::new(),
            combo: ComboSystem::new(),
            enemies: Vec::new(),
            invuln_timer: 0.0,
            level_time: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, px: f32, py: f32) {
        self.particles.update(dt);
        self.effects.update(dt);
        self.dash.update(dt, px, py);
        self.combo.update(dt);
        for e in &mut self.enemies { e.update(dt); }
        if self.invuln_timer > 0.0 { self.invuln_timer -= dt; }
        self.level_time += dt;
    }

    pub fn check_enemy_collision(&self, player: GeoPoint) -> bool {
        if self.invuln_timer > 0.0 { return false; }
        for e in &self.enemies {
            if e.distance_to_player(player) < e.radius + 0.015 { return true; }
        }
        false
    }

    pub fn reset(&mut self) {
        self.particles.clear();
        self.effects = ScreenEffects::new();
        self.dash = DashState::new();
        self.combo = ComboSystem::new();
        self.invuln_timer = 0.0;
        self.level_time = 0.0;
    }
}

// ==================== LEVEL TRAIT ====================

pub trait Level {
    fn geometry_type(&self) -> GeometryType;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn objective(&self) -> &str;
    fn collected_fraction(&self) -> f32;
    fn reset(&mut self);
    fn update(&mut self, dt: f32, move_x: f32, move_y: f32, action: bool, dash: bool) -> LevelResult;
    fn draw(&self);
}