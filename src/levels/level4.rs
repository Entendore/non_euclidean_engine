use macroquad::prelude::*;
use crate::geometry::{GeoPoint, GeometryType, hyperbolic::HyperbolicSpace};
use crate::engine::game::{Level, LevelResult, LevelRuntime, Wall, Portal, PortalType, Enemy, SizeIllusionPair};
use crate::engine::wall_builder::WallBuilder;
use crate::engine::collectible_manager::CollectibleManager;
use crate::engine::portal_system::PortalSystem;
use crate::engine::trail_system::TrailSystem;
use crate::engine::trigger_system::{TriggerZone, TriggerEvent, TriggerAction, TriggerSystem};
use crate::engine::renderer;
use crate::engine::illusion;

/// Level 4: The Infinite Maze
/// A complex hyperbolic maze built entirely from engine primitives.
/// Features: maze walls, portals, triggers, trail system, size illusions, enemies.
///
/// Uses: WallBuilder, CollectibleManager, PortalSystem, TrailSystem, TriggerSystem
pub struct InfiniteMaze {
    hyperbolic: HyperbolicSpace,
    player: GeoPoint,
    zoom: f32,
    completed: bool,
    completion_timer: f32,
    pub runtime: LevelRuntime,
    collectibles: CollectibleManager,
    portals: PortalSystem,
    walls: Vec<Wall>,
    trail: TrailSystem,
    triggers: TriggerSystem,
    size_illusions: Vec<SizeIllusionPair>,
    explored_depth: f32,
    max_depth_reached: f32,
    hint_timer: f32,
}

impl InfiniteMaze {
    pub fn new() -> Self {
        let hyperbolic = HyperbolicSpace::new(-1.0);

        // === COLLECTIBLES using CollectibleManager ===
        let mut collectibles = CollectibleManager::new(120);
        let crystal_labels = "ABCDEFGHIJKLMNOPQRSTUVWX";
        let crystal_colors = [
            YELLOW, Color::new(1.0, 0.8, 0.0, 1.0), Color::new(0.0, 1.0, 0.8, 1.0),
            Color::new(1.0, 0.5, 0.0, 1.0), Color::new(0.5, 1.0, 0.5, 1.0),
            Color::new(1.0, 1.0, 0.5, 1.0), Color::new(0.5, 0.8, 1.0, 1.0),
        ];

        // Place collectibles at strategic maze positions
        let positions = [
            (0.0, 0.08), (0.08, 0.0), (-0.08, 0.0), (0.0, -0.08),
            (0.2, 0.15), (-0.2, 0.15), (0.2, -0.15), (-0.2, -0.15),
            (0.0, 0.28), (0.28, 0.0), (-0.28, 0.0), (0.0, -0.28),
            (0.35, 0.25), (-0.35, 0.25), (0.35, -0.25), (-0.35, -0.25),
            (0.15, 0.45), (-0.15, 0.45), (0.15, -0.45), (-0.15, -0.45),
            (0.0, 0.55), (0.5, 0.15), (-0.5, 0.15), (0.5, -0.15),
        ];

        for (i, &(x, y)) in positions.iter().enumerate() {
            let label = crystal_labels.as_bytes()[i % crystal_labels.len()] as char;
            let color = crystal_colors[i % crystal_colors.len()];
            collectibles.add(crate::engine::game::Collectible::new(x, y, label, color));
        }

        // === WALLS using WallBuilder ===
        let mut walls = Vec::new();

        // Outer maze boundary ring at r=0.85
        walls.extend(WallBuilder::ring(0.0, 0.0, 0.85, 48,
            &[0.3, 1.6, 2.8, 4.1, 5.4], 0.3,
            Color::new(0.4, 0.2, 0.8, 1.0)));

        // Inner maze rings
        walls.extend(WallBuilder::concentric_rings(0.0, 0.0,
            &[0.15, 0.30, 0.45, 0.60, 0.75], 36, 4,
            Color::new(0.2, 0.35, 0.7, 1.0)));

        // Radial maze walls (spokes) — but with gaps to create passages
        for i in 0..12 {
            let base_angle = std::f32::consts::TAU * i as f32 / 12.0;
            // Only add some spokes to create maze-like passages
            if i % 2 == 0 {
                // Full spoke with a gap in the middle
                walls.push(Wall::new(
                    0.05 * base_angle.cos(), 0.05 * base_angle.sin(),
                    0.20 * base_angle.cos(), 0.20 * base_angle.sin(),
                    Color::new(0.15, 0.25, 0.55, 1.0)));
                walls.push(Wall::new(
                    0.28 * base_angle.cos(), 0.28 * base_angle.sin(),
                    0.42 * base_angle.cos(), 0.42 * base_angle.sin(),
                    Color::new(0.15, 0.25, 0.55, 1.0)));
            } else {
                // Partial spoke
                walls.push(Wall::new(
                    0.18 * base_angle.cos(), 0.18 * base_angle.sin(),
                    0.35 * base_angle.cos(), 0.35 * base_angle.sin(),
                    Color::new(0.12, 0.22, 0.5, 1.0)));
                walls.push(Wall::new(
                    0.48 * base_angle.cos(), 0.48 * base_angle.sin(),
                    0.65 * base_angle.cos(), 0.65 * base_angle.sin(),
                    Color::new(0.12, 0.22, 0.5, 1.0)));
            }
        }

        // Extra maze walls for complexity
        for i in 0..6 {
            let angle = std::f32::consts::FRAC_PI_3 * i as f32 + 0.25;
            walls.push(Wall::new(
                0.30 * angle.cos(), 0.30 * angle.sin(),
                0.55 * (angle + 0.2).cos(), 0.55 * (angle + 0.2).sin(),
                Color::new(0.18, 0.3, 0.6, 1.0)));
        }

        // === PORTALS using PortalSystem ===
        let portal_sys = PortalSystem::with_portals(vec![
            // Portal 0: Normal portal — shortcut from inner ring to outer ring
            {
                let mut p = Portal::with_type(0.12, 0.12, 0.55, 0.55, PortalType::Normal);
                p.radius = 0.035;
                p.show_geodesic_link = true;
                p.label = "N".to_string();
                p
            },
            // Portal 1: Auto-warp portal — walk through shortcut
            {
                let mut p = Portal::with_type(-0.5, 0.0, 0.0, 0.5, PortalType::AutoWarp);
                p.radius = 0.03;
                p.show_geodesic_link = true;
                p.show_corridor = true;
                p.label = "W".to_string();
                p
            },
            // Portal 2: Mirror portal
            {
                let mut p = Portal::with_type(-0.3, -0.3, 0.3, 0.3, PortalType::Mirror);
                p.radius = 0.03;
                p.show_geodesic_link = true;
                p.label = "M".to_string();
                p
            },
            // Portal 3: One-way shortcut
            {
                let mut p = Portal::with_type(0.0, -0.6, 0.6, 0.0, PortalType::OneWayAB);
                p.radius = 0.035;
                p.show_window = true;
                p.show_geodesic_link = true;
                p.label = ">".to_string();
                p
            },
        ]);

        // === TRIGGERS using TriggerSystem ===
        let mut triggers = TriggerSystem::new();

        // Center trigger: hint message on first visit
        triggers.add(TriggerZone::new(0.0, 0.0, 0.1,
            TriggerEvent::OnEnter,
            TriggerAction::Message("Deeper = harder! Watch for portals.".to_string(), 3.0)
        ).one_shot());

        // Depth trigger at r=0.5
        triggers.add(TriggerZone::new(0.0, 0.0, 0.55,
            TriggerEvent::OnEnter,
            TriggerAction::ScreenFlash(Color::new(0.3, 0.3, 1.0, 1.0), 0.2)
        ));

        // Outer ring warning
        triggers.add(TriggerZone::new(0.0, 0.0, 0.75,
            TriggerEvent::OnEnter,
            TriggerAction::Message("Edge of the maze — space warps here!".to_string(), 2.5)
        ).one_shot());

        // === TRAIL SYSTEM ===
        let trail = TrailSystem::new(Color::new(0.0, 1.0, 0.5, 1.0), 60);

        // === ENEMIES ===
        let mut runtime = LevelRuntime::new();
        runtime.enemies.push(Enemy::new(0.1, 0.3, -0.1, -0.3, 0.08));
        runtime.enemies.push(Enemy::new(-0.3, 0.1, 0.3, -0.1, 0.07));
        runtime.enemies.push(Enemy::new(0.0, -0.2, 0.0, -0.55, 0.06));
        runtime.enemies.push(Enemy::new(0.4, 0.0, -0.4, 0.0, 0.09));
        runtime.enemies.push(Enemy::new(0.2, 0.4, -0.2, 0.4, 0.07));

        // === SIZE ILLUSIONS ===
        let size_illusions = vec![
            SizeIllusionPair::new(0.0, 0.4, 0.65, 0.0, 0.03,
                Color::new(0.3, 0.8, 1.0, 0.8), "center vs edge"),
            SizeIllusionPair::new(-0.3, -0.5, 0.0, -0.7, 0.025,
                Color::new(1.0, 0.5, 0.3, 0.7), "near vs far"),
        ];

        Self {
            hyperbolic, player: GeoPoint::new(0.0, 0.0),
            zoom: 1.8, completed: false, completion_timer: 0.0,
            runtime, collectibles, portals: portal_sys,
            walls, trail, triggers, size_illusions,
            explored_depth: 0.0, max_depth_reached: 0.0,
            hint_timer: 0.0,
        }
    }
}

impl Level for InfiniteMaze {
    fn geometry_type(&self) -> GeometryType { GeometryType::Hyperbolic }
    fn name(&self) -> &str { "Level 4: The Infinite Maze" }
    fn description(&self) -> &str {
        "A hyperbolic maze built from engine primitives! Navigate concentric rings, \
         use portals (N=Normal, W=AutoWarp, M=Mirror, >=OneWay), avoid 5 enemies. \
         24 crystals hidden in the maze. Space warps near the edge!"
    }
    fn objective(&self) -> &str { "Collect all 24 crystals (A-X)" }

    fn collected_fraction(&self) -> f32 {
        self.collectibles.fraction()
    }

    fn reset(&mut self) { *self = Self::new(); }

    fn update(&mut self, dt: f32, move_x: f32, move_y: f32, action: bool, dash: bool) -> LevelResult {
        if dash { let _ = self.runtime.dash.try_dash(); }
        self.runtime.update(dt, self.player.x, self.player.y);

        if self.completed {
            self.completion_timer += dt;
            if self.completion_timer > 2.0 {
                return LevelResult::Completed(
                    self.collectibles.collected_count as u32 * 120
                    + self.runtime.combo.total_bonus);
            }
            return LevelResult::Playing;
        }

        // Trail
        self.trail.update(self.player.x, self.player.y, dt);

        // Enemy collision
        if self.runtime.check_enemy_collision(self.player) {
            self.runtime.effects.shake(8.0, 0.3);
            self.runtime.effects.flash(Color::new(1.0, 0.0, 0.0, 1.0), 0.2);
            self.runtime.invuln_timer = 1.5;
            self.runtime.combo.combo = 0;
            self.runtime.particles.emit_burst(self.player.x, self.player.y, RED, 20);
            // Knockback toward center
            let r = (self.player.x * self.player.x + self.player.y * self.player.y).sqrt();
            if r > 0.01 {
                self.player.x -= self.player.x / r * 0.04;
                self.player.y -= self.player.y / r * 0.04;
            }
        }

        // Movement with hyperbolic geometry
        let speed = 0.003 * self.runtime.dash.speed_multiplier();
        self.player = self.hyperbolic.move_point(self.player, move_x, move_y, speed);

        // Track exploration depth
        let r = (self.player.x * self.player.x + self.player.y * self.player.y).sqrt();
        if r > self.max_depth_reached {
            self.max_depth_reached = r;
        }

        // Portal system
        self.portals.update(dt);
        self.portals.try_teleport(&mut self.player, action, &mut self.runtime);

        // Collectibles
        self.collectibles.update(dt);
        self.collectibles.check_collection(
            self.player,
            |p, c| self.hyperbolic.distance(p, c),
            &mut self.runtime);

        // Triggers
        self.triggers.update(self.player, dt);
        for action in &self.triggers.pending_actions {
            match action {
                TriggerAction::Message(_msg, dur) => {
                    self.hint_timer = *dur as f32;
                }
                TriggerAction::ScreenFlash(color, dur) => {
                    self.runtime.effects.flash(*color, *dur);
                }
                TriggerAction::ScreenShake(intensity, dur) => {
                    self.runtime.effects.shake(*intensity, *dur);
                }
                TriggerAction::ParticleBurst(color, count) => {
                    self.runtime.particles.emit_burst(self.player.x, self.player.y, *color, *count);
                }
                _ => {}
            }
        }

        // Check completion
        if self.collectibles.all_collected() && !self.completed {
            self.completed = true;
            self.completion_timer = 0.0;
            self.runtime.effects.flash(YELLOW, 0.5);
            self.runtime.effects.shake(6.0, 0.5);
        }

        // Adaptive zoom — zoom out as player goes deeper
        self.zoom = 1.8 + r * 2.0;

        // Hint timer
        if self.hint_timer > 0.0 { self.hint_timer -= dt; }

        LevelResult::Playing
    }

    fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        let cx = self.player.x;
        let cy = self.player.y;
        let t = self.runtime.level_time;

        // Background — darker near edge
        let r = (self.player.x * self.player.x + self.player.y * self.player.y).sqrt();
        let bg_intensity = 0.02 + r * 0.03;
        clear_background(Color::new(bg_intensity * 0.5, bg_intensity * 0.5, bg_intensity * 2.0, 1.0));

        // Boundary circle
        renderer::draw_circle_outline_world(0.0, 0.0, 1.0, cx, cy, self.zoom, sw, sh,
            Color::new(0.5, 0.1, 0.2, 0.8), 3.0);
        // Pulsing boundary warning
        let pulse = (t * 2.0).sin() * 0.15 + 0.85;
        renderer::draw_circle_outline_world(0.0, 0.0, 0.99, cx, cy, self.zoom, sw, sh,
            Color::new(0.8, 0.2, 0.2, 0.3 * pulse), 1.5);

        // === GEODESIC GRID (conformal) ===
        let grid_color = Color::new(0.06, 0.08, 0.2, 0.4);
        for ring in 1..=10 {
            let gr = ring as f32 * 0.1;
            if gr < 0.98 {
                illusion::draw_geodesic_ring_conformal(0.0, 0.0, gr, 20,
                    cx, cy, self.zoom, sw, sh, grid_color, 0.8, 6);
            }
        }
        for i in 0..6 {
            let angle = std::f32::consts::TAU * i as f32 / 6.0;
            illusion::draw_geodesic_radial_conformal(0.0, 0.0, angle, 0.02, 0.98,
                cx, cy, self.zoom, sw, sh, grid_color, 0.8);
        }

        // === WALLS (as geodesic arcs with conformal scaling) ===
        for wall in &self.walls {
            illusion::draw_geodesic_line_conformal(
                wall.p1.x, wall.p1.y, wall.p2.x, wall.p2.y,
                cx, cy, self.zoom, sw, sh, wall.color, 2.5, 6);
        }

        // === TRIGGERS (debug visualization) ===
        self.triggers.draw(cx, cy, self.zoom, sw, sh);

        // === PORTALS ===
        self.portals.draw(cx, cy, self.zoom, sw, sh, t,
            // We need a PortalIllusion — create a temporary one
            // Actually let's use a static one
            &illusion::PortalIllusion::new());

        // === SIZE ILLUSIONS ===
        for pair in &self.size_illusions {
            renderer::draw_size_illusion_pair(
                pair.pos_a.x, pair.pos_a.y, pair.pos_b.x, pair.pos_b.y,
                pair.radius, cx, cy, self.zoom, sw, sh,
                pair.color, "A", "B", true);
        }

        // === CURVATURE FIELD ===
        illusion::draw_curvature_field(0.0, 0.0, 0.4, -1.0,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.2, 0.4, 1.0, 0.2), t);

        // === PARALLEL LINE DEVIATION ===
        illusion::draw_parallel_deviation(
            0.4, 0.6, -std::f32::consts::FRAC_PI_2, 0.05, 0.2, -0.8,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.3, 0.6, 1.0, 0.5), t);

        // === ENEMIES (conformal) ===
        for e in &self.runtime.enemies {
            let r2 = e.pos.x * e.pos.x + e.pos.y * e.pos.y;
            let _conformal = if r2 < 1.0 { (1.0 - r2) / 2.0 } else { 0.01 };
            let pulse = (e.phase).sin() * 0.005;
            renderer::draw_circle_world_conformal(e.pos.x, e.pos.y, e.alert_radius,
                cx, cy, self.zoom, sw, sh, Color::new(1.0, 0.0, 0.0, 0.06));
            renderer::draw_circle_world_conformal(e.pos.x, e.pos.y, e.radius + 0.015 + pulse,
                cx, cy, self.zoom, sw, sh, Color::new(1.0, 0.1, 0.0, 0.15));
            renderer::draw_circle_world_conformal(e.pos.x, e.pos.y, e.radius + pulse,
                cx, cy, self.zoom, sw, sh, e.color);
            renderer::draw_circle_world_conformal(e.pos.x, e.pos.y, e.radius * 0.35,
                cx, cy, self.zoom, sw, sh, Color::new(1.0, 1.0, 0.5, 0.9));
        }

        // === COLLECTIBLES (conformal) ===
        for c in &self.collectibles.collectibles {
            if c.collected { continue; }
            let er = c.effective_radius();
            let by = c.bob_y();
            renderer::draw_circle_world_conformal(c.pos.x, c.pos.y + by, er * 2.0,
                cx, cy, self.zoom, sw, sh,
                Color::new(c.color.r, c.color.g, c.color.b, 0.2));
            renderer::draw_circle_world_conformal(c.pos.x, c.pos.y + by, er,
                cx, cy, self.zoom, sw, sh, c.color);
            renderer::draw_text_world_conformal(&c.label.to_string(),
                c.pos.x - 0.01, c.pos.y + by - 0.01,
                cx, cy, self.zoom, sw, sh, 0.04, WHITE);
        }

        // === TRAIL ===
        self.trail.draw(cx, cy, self.zoom, sw, sh);

        // === PLAYER ===
        let player_r = 0.02;
        let invuln_flash = if self.runtime.invuln_timer > 0.0 {
            (self.runtime.invuln_timer * 10.0).sin().abs() > 0.3
        } else { false };
        if !invuln_flash {
            renderer::draw_circle_world(self.player.x, self.player.y, player_r * 2.0,
                cx, cy, self.zoom, sw, sh, Color::new(0.0, 1.0, 0.5, 0.2));
            renderer::draw_circle_world(self.player.x, self.player.y, player_r,
                cx, cy, self.zoom, sw, sh, GREEN);
        }

        // === PARTICLES ===
        self.runtime.particles.draw(cx, cy, self.zoom, sw, sh);

        // === PROXIMITY INDICATORS ===
        self.collectibles.draw_proximity(cx, cy, self.zoom, sw, sh);

        // === HUD OVERLAYS ===
        // Depth indicator
        let depth_text = format!("Depth: {:.0}%", self.max_depth_reached * 100.0);
        renderer::draw_text_world(&depth_text, 0.0, 0.9,
            cx, cy, self.zoom, sw, sh, 0.02,
            Color::new(0.4, 0.6, 1.0, 0.5 + (t * 0.5).sin() * 0.15));

        // Engine primitives label
        renderer::draw_text_world("WALL_BUILDER | PORTAL_SYSTEM | TRIGGER_SYSTEM | TRAIL",
            0.0, -0.92, cx, cy, self.zoom, sw, sh, 0.015,
            Color::new(0.3, 0.4, 0.6, 0.3));

        // Completion flash
        if self.completed {
            let flash = (self.completion_timer * 5.0).sin().abs();
            renderer::draw_circle_world(self.player.x, self.player.y, 0.1 + flash * 0.05,
                cx, cy, self.zoom, sw, sh, Color::new(1.0, 1.0, 0.0, 0.3));
        }

        self.runtime.effects.draw_flash();
    }
}