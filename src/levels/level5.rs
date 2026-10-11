use macroquad::prelude::*;
use crate::geometry::{GeoPoint, GeometryType};
use crate::geometry::hyperspace::{
    NPoint, Hyperspherical4D, Hyperspherical5D,
    Hyperbolic4D, Hyperbolic5D,
};
use crate::engine::game::{Level, LevelResult, LevelRuntime, Collectible, Portal, PortalType, Enemy};
use crate::engine::portal_system::PortalSystem;
use crate::engine::collectible_manager::CollectibleManager;
use crate::engine::trail_system::TrailSystem;
use crate::engine::trigger_system::{TriggerZone, TriggerEvent, TriggerAction, TriggerSystem};
use crate::engine::renderer;
use crate::engine::illusion;

/// Which higher-dimensional geometry mode the player is currently in.
#[derive(Debug, Clone, Copy, PartialEq)]
enum DimMode {
    /// 4D Hyperspherical (S³) — walking on a 3-sphere
    Hyperspherical4D,
    /// 5D Hyperspherical (S⁴) — walking on a 4-sphere
    Hyperspherical5D,
    /// 4D Hyperbolic (H⁴) — Poincaré ball in 4D
    Hyperbolic4D,
    /// 5D Hyperbolic (H⁵) — Poincaré ball in 5D
    Hyperbolic5D,
}

impl DimMode {
    fn label(&self) -> &'static str {
        match self {
            DimMode::Hyperspherical4D => "4D Hypersphere (S3)",
            DimMode::Hyperspherical5D => "5D Hypersphere (S4)",
            DimMode::Hyperbolic4D => "4D Hyperbolic (H4)",
            DimMode::Hyperbolic5D => "5D Hyperbolic (H5)",
        }
    }

    fn dim_count(&self) -> usize {
        match self {
            DimMode::Hyperspherical4D | DimMode::Hyperbolic4D => 4,
            DimMode::Hyperspherical5D | DimMode::Hyperbolic5D => 5,
        }
    }

    fn is_positive_curvature(&self) -> bool {
        matches!(self, DimMode::Hyperspherical4D | DimMode::Hyperspherical5D)
    }

    fn color(&self) -> Color {
        match self {
            DimMode::Hyperspherical4D => Color::new(0.0, 0.8, 1.0, 1.0),
            DimMode::Hyperspherical5D => Color::new(1.0, 0.4, 0.0, 1.0),
            DimMode::Hyperbolic4D => Color::new(0.8, 0.0, 1.0, 1.0),
            DimMode::Hyperbolic5D => Color::new(0.0, 1.0, 0.4, 1.0),
        }
    }
}

/// Level 5: Beyond Dimensions
/// Navigates through 4D and 5D geometric spaces using stereographic/perspective
/// projection to display higher-dimensional geometry on screen.
///
/// The level cycles through 4 geometry modes:
/// 1. 4D Hyperspherical (S³) — positive curvature, finite, wraps around
/// 2. 5D Hyperspherical (S⁴) — stronger positive curvature
/// 3. 4D Hyperbolic (H⁴) — negative curvature, Poincaré ball in 4D
/// 4. 5D Hyperbolic (H⁵) — even stronger negative curvature
///
/// Uses engine primitives: PortalSystem, CollectibleManager, TrailSystem, TriggerSystem
pub struct BeyondDimensions {
    // Geometry engines
    sphere4d: Hyperspherical4D,
    sphere5d: Hyperspherical5D,
    hyp4d: Hyperbolic4D,
    hyp5d: Hyperbolic5D,

    // Player position (stored as N-dimensional point)
    player_nd: NPoint,
    player_2d: GeoPoint,  // Projected 2D position for rendering/collision
    dim_mode: DimMode,
    zoom: f32,

    // Collectibles (stored in 2D projected space for simplicity)
    collectibles: CollectibleManager,

    // Portals
    portals: PortalSystem,

    // Runtime
    pub runtime: LevelRuntime,
    trail: TrailSystem,
    triggers: TriggerSystem,

    // State
    completed: bool,
    completion_timer: f32,
    wrap_flash_timer: f32,
    dim_shift_timer: f32,
    extra_dim_offset: f32,
    hyperspace_scale: f32,
    hint_timer: f32,
    geo_demos: Vec<GeoDemo>,
}

/// A geometry demonstration point showing a property of the current space.
struct GeoDemo {
    pos_2d: GeoPoint,
    label: String,
    demo_type: GeoDemoType,
}

enum GeoDemoType {
    AngleSum,
    ParallelTransport,
    DimensionalAxis,
    VolumeGrowth,
}

impl BeyondDimensions {
    pub fn new() -> Self {
        let sphere4d = Hyperspherical4D::new(1.0);
        let sphere5d = Hyperspherical5D::new(1.0);
        let hyp4d = Hyperbolic4D::new(-1.0);
        let hyp5d = Hyperbolic5D::new(-1.0);

        // Start at the "north pole" of S³ (0,0,0,1)
        let player_nd = NPoint::new4(0.0, 0.0, 0.01, 0.99);
        let player_2d = GeoPoint::new(0.0, 0.0);

        // === COLLECTIBLES ===
        let mut collectibles = CollectibleManager::new(150);
        let labels = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmn";
        let colors = [
            Color::new(0.0, 0.9, 0.9, 1.0),  // cyan
            Color::new(1.0, 0.8, 0.0, 1.0),  // gold
            Color::new(1.0, 0.4, 0.0, 1.0),  // orange
            Color::new(0.5, 1.0, 0.5, 1.0),  // lime
            Color::new(1.0, 0.5, 0.8, 1.0),  // pink
            Color::new(0.7, 0.7, 1.0, 1.0),  // lavender
        ];

        // Place collectibles in a pattern that works across projections
        let positions_2d: [(f32, f32); 20] = [
            (0.0, 0.15), (0.0, -0.15), (0.15, 0.0), (-0.15, 0.0),
            (0.1, 0.1), (-0.1, 0.1), (0.1, -0.1), (-0.1, -0.1),
            (0.0, 0.3), (0.3, 0.0), (-0.3, 0.0), (0.0, -0.3),
            (0.2, 0.2), (-0.2, 0.2), (0.2, -0.2), (-0.2, -0.2),
            (0.0, 0.5), (0.5, 0.0), (-0.5, 0.0), (0.0, -0.5),
        ];

        for (i, &(x, y)) in positions_2d.iter().enumerate() {
            let label = labels.as_bytes()[i % labels.len()] as char;
            let color = colors[i % colors.len()];
            collectibles.add(Collectible::new(x, y, label, color));
        }

        // === PORTALS (dimensional shifts) ===
        let portal_sys = PortalSystem::with_portals(vec![
            // Portal 0: Shift from 4D Spherical to 5D Spherical
            {
                let mut p = Portal::with_type(0.35, 0.0, -0.35, 0.0, PortalType::DimensionalShift);
                p.radius = 0.05;
                p.show_geodesic_link = true;
                p.show_corridor = true;
                p.label = "4D>5D".to_string();
                p
            },
            // Portal 1: Shift from 5D Spherical to 4D Hyperbolic
            {
                let mut p = Portal::with_type(0.0, 0.35, 0.0, -0.35, PortalType::DimensionalShift);
                p.radius = 0.05;
                p.show_geodesic_link = true;
                p.show_corridor = true;
                p.label = "S>H".to_string();
                p
            },
            // Portal 2: Shift from 4D Hyperbolic to 5D Hyperbolic
            {
                let mut p = Portal::with_type(-0.3, 0.3, 0.3, -0.3, PortalType::DimensionalShift);
                p.radius = 0.05;
                p.show_geodesic_link = true;
                p.label = "4>5H".to_string();
                p
            },
            // Portal 3: Auto-warp shortcut
            {
                let mut p = Portal::with_type(0.4, 0.3, -0.4, -0.3, PortalType::AutoWarp);
                p.radius = 0.04;
                p.show_geodesic_link = true;
                p.show_corridor = true;
                p.label = "W".to_string();
                p
            },
        ]);

        // === TRIGGERS ===
        let mut triggers = TriggerSystem::new();
        triggers.add(TriggerZone::new(0.0, 0.0, 0.08,
            TriggerEvent::OnEnter,
            TriggerAction::Message("Welcome to 4D Hyperspherical space! Walk on the surface of a 3-sphere.".to_string(), 4.0)
        ).one_shot());

        triggers.add(TriggerZone::new(0.0, 0.0, 0.25,
            TriggerEvent::OnProximity(0.2),
            TriggerAction::ParticleBurst(Color::new(0.0, 0.8, 1.0, 1.0), 8)));

        // === TRAIL ===
        let trail = TrailSystem::new(Color::new(0.5, 0.5, 1.0, 1.0), 80);

        // === ENEMIES ===
        let mut runtime = LevelRuntime::new();
        runtime.enemies.push(Enemy::new(0.15, 0.15, -0.15, -0.15, 0.06));
        runtime.enemies.push(Enemy::new(-0.2, 0.0, 0.2, 0.0, 0.05));
        runtime.enemies.push(Enemy::new(0.0, 0.2, 0.0, -0.25, 0.07));

        // === GEOMETRY DEMOS ===
        let geo_demos = vec![
            GeoDemo {
                pos_2d: GeoPoint::new(0.4, 0.4),
                label: "Angle Sum > 180".to_string(),
                demo_type: GeoDemoType::AngleSum,
            },
            GeoDemo {
                pos_2d: GeoPoint::new(-0.4, 0.4),
                label: "Parallel Transport".to_string(),
                demo_type: GeoDemoType::ParallelTransport,
            },
            GeoDemo {
                pos_2d: GeoPoint::new(0.4, -0.4),
                label: "5th Axis (V)".to_string(),
                demo_type: GeoDemoType::DimensionalAxis,
            },
            GeoDemo {
                pos_2d: GeoPoint::new(-0.4, -0.4),
                label: "Volume Growth".to_string(),
                demo_type: GeoDemoType::VolumeGrowth,
            },
        ];

        Self {
            sphere4d, sphere5d, hyp4d, hyp5d,
            player_nd, player_2d,
            dim_mode: DimMode::Hyperspherical4D,
            zoom: 2.0,
            collectibles, portals: portal_sys,
            runtime, trail, triggers,
            completed: false, completion_timer: 0.0,
            wrap_flash_timer: 0.0, dim_shift_timer: 0.0,
            extra_dim_offset: 0.0, hyperspace_scale: 1.0,
            hint_timer: 0.0, geo_demos,
        }
    }

    /// Move the player in whatever geometry mode we're in.
    fn move_player(&mut self, dx: f32, dy: f32, speed: f32) {
        let old_nd = self.player_nd;

        match self.dim_mode {
            DimMode::Hyperspherical4D => {
                self.player_nd = self.sphere4d.move_point(&self.player_nd, dx, dy, speed);
                if self.sphere4d.did_wrap(&old_nd, &self.player_nd) {
                    self.wrap_flash_timer = 0.5;
                }
                self.player_2d = self.sphere4d.to_2d(&self.player_nd);
                self.hyperspace_scale = self.sphere4d.scale_at(&self.player_nd).min(3.0);
            }
            DimMode::Hyperspherical5D => {
                self.player_nd = self.sphere5d.move_point(&self.player_nd, dx, dy, speed);
                if self.sphere5d.did_wrap(&old_nd, &self.player_nd) {
                    self.wrap_flash_timer = 0.5;
                }
                self.player_2d = self.sphere5d.to_2d(&self.player_nd);
                self.hyperspace_scale = self.sphere5d.scale_at(&self.player_nd).min(3.0);
            }
            DimMode::Hyperbolic4D => {
                self.player_nd = self.hyp4d.move_point(&self.player_nd, dx, dy, speed);
                self.player_2d = self.hyp4d.to_2d(&self.player_nd);
                self.hyperspace_scale = self.hyp4d.scale_at(&self.player_nd).min(3.0).max(0.15);
            }
            DimMode::Hyperbolic5D => {
                self.player_nd = self.hyp5d.move_point(&self.player_nd, dx, dy, speed);
                self.player_2d = self.hyp5d.to_2d(&self.player_nd);
                self.hyperspace_scale = self.hyp5d.scale_at(&self.player_nd).min(3.0).max(0.15);
            }
        }

        // Compute extra-dimensional offset for visual effect
        match self.dim_mode {
            DimMode::Hyperspherical4D => {
                self.extra_dim_offset = self.player_nd.coords[2] * 0.3
                    + self.player_nd.coords[3] * 0.15;
            }
            DimMode::Hyperspherical5D => {
                self.extra_dim_offset = self.player_nd.coords[2] * 0.2
                    + self.player_nd.coords[3] * 0.2
                    + self.player_nd.coords[4] * 0.15;
            }
            DimMode::Hyperbolic4D => {
                self.extra_dim_offset = (self.player_nd.coords[2] + self.player_nd.coords[3]) * 0.2;
            }
            DimMode::Hyperbolic5D => {
                self.extra_dim_offset = (self.player_nd.coords[2] + self.player_nd.coords[3]
                    + self.player_nd.coords[4]) * 0.15;
            }
        }
    }

    /// Switch to a new dimension mode.
    fn switch_dimension(&mut self, new_mode: DimMode) {
        if self.dim_mode == new_mode { return; }

        // Re-initialize player position for new geometry
        match new_mode {
            DimMode::Hyperspherical4D => {
                self.player_nd = NPoint::new4(0.0, 0.0, 0.01, 0.99);
            }
            DimMode::Hyperspherical5D => {
                self.player_nd = NPoint::new5(0.0, 0.0, 0.01, 0.01, 0.99);
            }
            DimMode::Hyperbolic4D => {
                self.player_nd = NPoint::new4(0.0, 0.0, 0.0, 0.0);
            }
            DimMode::Hyperbolic5D => {
                self.player_nd = NPoint::new5(0.0, 0.0, 0.0, 0.0, 0.0);
            }
        }

        self.dim_mode = new_mode;
        self.dim_shift_timer = 1.0; // Trigger visual effect
        self.hint_timer = 3.0;
    }
}

impl Level for BeyondDimensions {
    fn geometry_type(&self) -> GeometryType {
        match self.dim_mode {
            DimMode::Hyperspherical4D => GeometryType::Hyperspherical4D,
            DimMode::Hyperspherical5D => GeometryType::Hyperspherical5D,
            DimMode::Hyperbolic4D => GeometryType::Hyperbolic4D,
            DimMode::Hyperbolic5D => GeometryType::Hyperbolic5D,
        }
    }

    fn name(&self) -> &str { "Level 5: Beyond Dimensions" }

    fn description(&self) -> &str {
        "Navigate 4D and 5D geometric spaces! Uses stereographic and perspective projections \
         to render higher-dimensional geometry. Collect 20 shards across 4 dimension modes. \
         D-portals shift between 4D Spherical, 5D Spherical, 4D Hyperbolic, and 5D Hyperbolic spaces."
    }

    fn objective(&self) -> &str {
        "Collect all 20 shards across 4 dimension modes (A-T)"
    }

    fn collected_fraction(&self) -> f32 {
        self.collectibles.fraction()
    }

    fn reset(&mut self) { *self = Self::new(); }

    fn update(&mut self, dt: f32, move_x: f32, move_y: f32, action: bool, dash: bool) -> LevelResult {
        if dash { let _ = self.runtime.dash.try_dash(); }
        self.runtime.update(dt, self.player_2d.x, self.player_2d.y);

        if self.completed {
            self.completion_timer += dt;
            if self.completion_timer > 2.0 {
                return LevelResult::Completed(
                    self.collectibles.collected_count as u32 * 150
                    + self.runtime.combo.total_bonus);
            }
            return LevelResult::Playing;
        }

        // Trail
        self.trail.update(self.player_2d.x, self.player_2d.y, dt);

        // Timers
        if self.wrap_flash_timer > 0.0 { self.wrap_flash_timer -= dt; }
        if self.dim_shift_timer > 0.0 { self.dim_shift_timer -= dt; }
        if self.hint_timer > 0.0 { self.hint_timer -= dt; }

        // Enemy collision
        if self.runtime.check_enemy_collision(self.player_2d) {
            self.runtime.effects.shake(8.0, 0.3);
            self.runtime.effects.flash(Color::new(1.0, 0.0, 0.0, 1.0), 0.2);
            self.runtime.invuln_timer = 1.5;
            self.runtime.combo.combo = 0;
            self.runtime.particles.emit_burst(self.player_2d.x, self.player_2d.y, RED, 20);
            // Knockback
            let r = (self.player_2d.x * self.player_2d.x + self.player_2d.y * self.player_2d.y).sqrt();
            if r > 0.01 {
                let factor = if self.dim_mode.is_positive_curvature() { -0.05 } else { -0.03 };
                self.player_2d.x += self.player_2d.x / r * factor;
                self.player_2d.y += self.player_2d.y / r * factor;
            }
        }

        // Movement in higher-dimensional space
        let speed = 0.004 * self.runtime.dash.speed_multiplier();
        self.move_player(move_x, move_y, speed);

        // Portal system — check for dimensional shifts
        self.portals.update(dt);
        let _old_mode = self.dim_mode;
        self.portals.try_teleport(&mut self.player_2d, action, &mut self.runtime);

        // Check if a DimensionalShift portal was used — cycle to next mode
        for portal in &self.portals.portals {
            if portal.portal_type == PortalType::DimensionalShift && portal.cooldown > 0.9 {
                let next = match self.dim_mode {
                    DimMode::Hyperspherical4D => DimMode::Hyperspherical5D,
                    DimMode::Hyperspherical5D => DimMode::Hyperbolic4D,
                    DimMode::Hyperbolic4D => DimMode::Hyperbolic5D,
                    DimMode::Hyperbolic5D => DimMode::Hyperspherical4D,
                };
                self.switch_dimension(next);
                break;
            }
        }

        // Collectibles (use 2D projected distance)
        self.collectibles.update(dt);
        self.collectibles.check_collection(
            self.player_2d,
            |p, c| p.distance_to(c),
            &mut self.runtime);

        // Triggers
        self.triggers.update(self.player_2d, dt);
        for act in &self.triggers.pending_actions {
            match act {
                TriggerAction::Message(_msg, dur) => {
                    self.hint_timer = *dur as f32;
                    self.runtime.particles.emit_burst(self.player_2d.x, self.player_2d.y,
                        self.dim_mode.color(), 5);
                }
                TriggerAction::ParticleBurst(color, count) => {
                    self.runtime.particles.emit_burst(self.player_2d.x, self.player_2d.y, *color, *count);
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

        // Adaptive zoom based on geometry
        let r2 = self.player_2d.x * self.player_2d.x + self.player_2d.y * self.player_2d.y;
        self.zoom = if self.dim_mode.is_positive_curvature() {
            2.0 + r2 * 0.5
        } else {
            2.0 + r2 * 2.5
        };

        LevelResult::Playing
    }

    fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        let cx = self.player_2d.x;
        let cy = self.player_2d.y;
        let t = self.runtime.level_time;
        let mode_color = self.dim_mode.color();

        // Background — changes with dimension mode
        let bg_r = if self.dim_mode.is_positive_curvature() { 0.04 } else { 0.01 };
        let bg_g = if self.dim_mode.is_positive_curvature() { 0.02 } else { 0.01 };
        let bg_b = 0.06 + (t * 0.3).sin() * 0.01;
        clear_background(Color::new(bg_r, bg_g, bg_b, 1.0));

        // Dimension shift visual effect
        if self.dim_shift_timer > 0.0 {
            renderer::draw_dimension_shift_effect(sw, sh, self.dim_shift_timer, mode_color, t);
        }

        // Wrap flash (for spherical spaces)
        if self.wrap_flash_timer > 0.0 {
            let alpha = self.wrap_flash_timer * 0.5;
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(1.0, 1.0, 1.0, alpha));
        }

        // === HYPERSPACE GRID ===
        let grid_r = if self.dim_mode.is_positive_curvature() { 0.7 } else { 0.9 };
        renderer::draw_hyperspace_grid(0.0, 0.0, grid_r, cx, cy, self.zoom, sw, sh,
            Color::new(mode_color.r, mode_color.g, mode_color.b, 0.4),
            t, self.dim_mode.dim_count());

        // === BOUNDARY (for hyperbolic modes) ===
        if !self.dim_mode.is_positive_curvature() {
            let boundary_alpha = 0.4 + (t * 2.0).sin() * 0.15;
            renderer::draw_circle_outline_world(0.0, 0.0, 1.0, cx, cy, self.zoom, sw, sh,
                Color::new(0.8, 0.2, 0.2, boundary_alpha), 2.5);
            // Warning ring
            renderer::draw_circle_outline_world(0.0, 0.0, 0.95, cx, cy, self.zoom, sw, sh,
                Color::new(1.0, 0.3, 0.1, 0.15), 1.0);
        }

        // === TESSERACT / PENTERACT DECORATIONS ===
        // Draw rotating wireframes as dimensional landmarks
        let deco_positions = [(0.5, 0.0), (-0.5, 0.0), (0.0, 0.5), (0.0, -0.5)];
        for (i, &(dx, dy)) in deco_positions.iter().enumerate() {
            let rot = t * 0.8 + i as f32 * 1.5;
            if self.dim_mode.dim_count() >= 5 {
                renderer::draw_penteract_wireframe(dx, dy, 0.06, cx, cy, self.zoom, sw, sh,
                    Color::new(mode_color.r, mode_color.g, mode_color.b, 0.3), rot);
            } else {
                renderer::draw_tesseract_wireframe(dx, dy, 0.06, cx, cy, self.zoom, sw, sh,
                    Color::new(mode_color.r, mode_color.g, mode_color.b, 0.3), rot);
            }
        }

        // === TRIGGERS ===
        self.triggers.draw(cx, cy, self.zoom, sw, sh);

        // === PORTALS ===
        self.portals.draw(cx, cy, self.zoom, sw, sh, t, &illusion::PortalIllusion::new());

        // === GEOMETRY DEMOS ===
        for demo in &self.geo_demos {
            let dx = demo.pos_2d.x - cx;
            let dy = demo.pos_2d.y - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 0.8 {
                let alpha = (0.4 - dist * 0.3).max(0.1);
                renderer::draw_text_world(&demo.label,
                    demo.pos_2d.x - 0.06, demo.pos_2d.y - 0.01,
                    cx, cy, self.zoom, sw, sh, 0.015,
                    Color::new(1.0, 0.8, 0.5, alpha));
            }
        }

        // === ENEMIES (with hyperspace scaling) ===
        for e in &self.runtime.enemies {
            let pulse = (e.phase).sin() * 0.005;
            renderer::draw_circle_world_hyperspace(
                e.pos.x, e.pos.y, e.alert_radius,
                cx, cy, self.zoom, sw, sh,
                Color::new(1.0, 0.0, 0.0, 0.06),
                self.hyperspace_scale, self.extra_dim_offset);
            renderer::draw_circle_world_hyperspace(
                e.pos.x, e.pos.y, e.radius + pulse,
                cx, cy, self.zoom, sw, sh, e.color,
                self.hyperspace_scale, self.extra_dim_offset);
            renderer::draw_circle_world_hyperspace(
                e.pos.x, e.pos.y, e.radius * 0.35,
                cx, cy, self.zoom, sw, sh,
                Color::new(1.0, 1.0, 0.5, 0.9),
                self.hyperspace_scale, self.extra_dim_offset);

            // Dimensional ghost (shows enemy position in adjacent dimension)
            if self.dim_mode.dim_count() >= 4 {
                let ghost_offset_x = (t * 2.0 + e.phase).sin() * 0.08;
                let ghost_offset_y = (t * 1.5 + e.phase).cos() * 0.08;
                renderer::draw_dimensional_ghost(
                    e.pos.x, e.pos.y, e.radius,
                    cx, cy, self.zoom, sw, sh,
                    e.color, ghost_offset_x, ghost_offset_y, 0.15);
            }
        }

        // === COLLECTIBLES (with hyperspace scaling + dimensional ghosts) ===
        for c in &self.collectibles.collectibles {
            if c.collected { continue; }
            let er = c.effective_radius();
            let by = c.bob_y();

            // Main collectible
            renderer::draw_circle_world_hyperspace(
                c.pos.x, c.pos.y + by, er * 2.0,
                cx, cy, self.zoom, sw, sh,
                Color::new(c.color.r, c.color.g, c.color.b, 0.2),
                self.hyperspace_scale, self.extra_dim_offset);
            renderer::draw_circle_world_hyperspace(
                c.pos.x, c.pos.y + by, er,
                cx, cy, self.zoom, sw, sh, c.color,
                self.hyperspace_scale, self.extra_dim_offset);

            // Label
            renderer::draw_text_world_hyperspace(
                &c.label.to_string(),
                c.pos.x - 0.01, c.pos.y + by - 0.01,
                cx, cy, self.zoom, sw, sh, 0.04, WHITE,
                self.hyperspace_scale, self.extra_dim_offset);

            // Dimensional ghost
            if self.dim_mode.dim_count() >= 5 {
                let ghost_dx = (t * 1.8 + c.pulse_phase).sin() * 0.06;
                let ghost_dy = (t * 1.3 + c.bob_offset).cos() * 0.06;
                renderer::draw_dimensional_ghost(
                    c.pos.x, c.pos.y, er * 0.7,
                    cx, cy, self.zoom, sw, sh,
                    c.color, ghost_dx, ghost_dy, 0.12);
            }
        }

        // === TRAIL ===
        self.trail.draw(cx, cy, self.zoom, sw, sh);

        // === PLAYER ===
        let invuln_flash = if self.runtime.invuln_timer > 0.0 {
            (self.runtime.invuln_timer * 10.0).sin().abs() > 0.3
        } else { false };

        if !invuln_flash {
            // Player glow
            renderer::draw_circle_world_hyperspace(
                self.player_2d.x, self.player_2d.y, 0.04,
                cx, cy, self.zoom, sw, sh,
                Color::new(0.0, 1.0, 0.5, 0.2),
                self.hyperspace_scale, 0.0);
            // Player body
            renderer::draw_circle_world_hyperspace(
                self.player_2d.x, self.player_2d.y, 0.02,
                cx, cy, self.zoom, sw, sh, GREEN,
                self.hyperspace_scale, 0.0);

            // Dimensional ghost of player
            let p_ghost_x = (t * 1.5).sin() * 0.05;
            let p_ghost_y = (t * 1.2).cos() * 0.05;
            renderer::draw_dimensional_ghost(
                self.player_2d.x, self.player_2d.y, 0.018,
                cx, cy, self.zoom, sw, sh, GREEN,
                p_ghost_x, p_ghost_y, 0.25);
        }

        // === PARTICLES ===
        self.runtime.particles.draw(cx, cy, self.zoom, sw, sh);

        // === PROXIMITY INDICATORS ===
        self.collectibles.draw_proximity(cx, cy, self.zoom, sw, sh);

        // === HUD OVERLAYS ===
        // Current dimension mode label
        let mode_text = self.dim_mode.label();
        renderer::draw_text_world(mode_text, 0.0, 0.8,
            cx, cy, self.zoom, sw, sh, 0.025, mode_color);

        // Dimension count
        let dim_text = format!("Dimensions: {}D", self.dim_mode.dim_count());
        renderer::draw_text_world(&dim_text, 0.0, 0.72,
            cx, cy, self.zoom, sw, sh, 0.018,
            Color::new(mode_color.r, mode_color.g, mode_color.b, 0.6));

        // Curvature type
        let curve_text = if self.dim_mode.is_positive_curvature() {
            "Positive Curvature (finite, wraps around)"
        } else {
            "Negative Curvature (infinite, boundary)"
        };
        renderer::draw_text_world(curve_text, 0.0, -0.85,
            cx, cy, self.zoom, sw, sh, 0.015,
            Color::new(0.6, 0.6, 0.7, 0.5));

        // Extra-dimensional coordinates display
        let coord_text = if self.player_nd.dims >= 4 {
            format!("({}, {}, {}, {})",
                self.player_nd.coords[0].to_degrees().abs().round(),
                self.player_nd.coords[1].to_degrees().abs().round(),
                self.player_nd.coords[2].to_degrees().abs().round(),
                self.player_nd.coords[3].to_degrees().abs().round())
        } else {
            "N/A".to_string()
        };
        let coord_5d = if self.player_nd.dims >= 5 {
            format!(" + V={}", self.player_nd.coords[4].to_degrees().abs().round())
        } else {
            String::new()
        };
        renderer::draw_text_world(&format!("Coords: {}{}", coord_text, coord_5d),
            0.0, -0.92, cx, cy, self.zoom, sw, sh, 0.013,
            Color::new(0.5, 0.5, 0.6, 0.4));

        // Hyperspace scale indicator
        let scale_text = format!("Proj. Scale: {:.2}x", self.hyperspace_scale);
        renderer::draw_text_world(&scale_text, 0.0, -0.99,
            cx, cy, self.zoom, sw, sh, 0.013,
            Color::new(0.4, 0.6, 0.8, 0.35));

        // Engine primitives label
        renderer::draw_text_world(
            "4D/5D GEOMETRY ENGINE | CROSS-SECTIONS | PENTERACT VISUALIZATION",
            0.0, -1.06, cx, cy, self.zoom, sw, sh, 0.012,
            Color::new(0.3, 0.4, 0.6, 0.25));

        // Cross-section visualization
        let slice_axis = if self.player_nd.dims >= 5 { 4 } else { 3 };
        let slice_coord = self.player_nd.coords[slice_axis];
        renderer::draw_cross_section(
            0.5, -0.5, 0.2, slice_coord, slice_axis,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.8, 0.4, 0.0, 1.0), t);

        // Rotation plane indicator
        renderer::draw_rotation_plane(
            -0.5, 0.5, 0.15,
            if self.player_nd.dims >= 4 { "X" } else { "X" },
            if self.player_nd.dims >= 4 { "W" } else { "Y" },
            t * 0.5,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.5, 0.3, 0.8, 1.0),
        );

        // Penteract decoration (new 5D hypercube)
        illusion::draw_penteract(
            0.5, 0.5, 0.12, cx, cy, self.zoom, sw, sh,
            Color::new(1.0, 0.5, 0.0, 0.7), t);

        // Klein bottle decoration (impossible 4D surface)
        illusion::draw_klein_bottle(
            -0.5, -0.5, 0.12, cx, cy, self.zoom, sw, sh,
            Color::new(0.0, 0.8, 0.5, 0.6), t);

        // Mirror room illusion
        let curvature_val = if self.dim_mode.is_positive_curvature() { 1.0 } else { -1.0 };
        illusion::draw_mirror_room(
            0.0, 0.5, 0.15, cx, cy, self.zoom, sw, sh,
            Color::new(0.5, 0.3, 0.8, 0.5), t, curvature_val);

        // Hint message
        if self.hint_timer > 0.0 {
            let hint_alpha = self.hint_timer.min(1.0);
            let hint = match self.dim_mode {
                DimMode::Hyperspherical4D => "You are on the 3-sphere (S3). Walking far enough wraps you around!",
                DimMode::Hyperspherical5D => "5D hyperspherical space (S4). Even more curvature than 4D!",
                DimMode::Hyperbolic4D => "4D hyperbolic space (H4). Infinite room inside this ball!",
                DimMode::Hyperbolic5D => "5D hyperbolic space (H5). The boundary is infinitely far away!",
            };
            renderer::draw_text_world(hint,
                0.0, 0.65, cx, cy, self.zoom, sw, sh, 0.016,
                Color::new(1.0, 1.0, 0.8, hint_alpha * 0.7));
        }

        // Completion flash
        if self.completed {
            let flash = (self.completion_timer * 5.0).sin().abs();
            renderer::draw_circle_world(self.player_2d.x, self.player_2d.y, 0.1 + flash * 0.05,
                cx, cy, self.zoom, sw, sh, Color::new(1.0, 1.0, 0.0, 0.3));
        }

        self.runtime.effects.draw_flash();
    }
}