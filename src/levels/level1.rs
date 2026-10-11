use macroquad::prelude::*;
use crate::geometry::{GeoPoint, GeometryType, hyperbolic::HyperbolicSpace};
use crate::engine::game::{Level, LevelResult, LevelRuntime, Collectible, Wall, Enemy, SizeIllusionPair, CurvatureZone};
use crate::engine::renderer;
use crate::engine::illusion;

/// Level 1: Hyperbolic Corridors
/// Set in the Poincaré disk model with full conformal rendering.
///
/// ENHANCED with engine features:
/// - Conformal scaling: objects near the disk edge appear smaller (defining hyperbolic property)
/// - Curvature field visualization: arrows showing space diverging
/// - Parallel line deviation demo: shows lines diverging in hyperbolic space
/// - Size illusion pairs: same-size objects look different at different radii
/// - Geodesic grid with conformal thickness
/// - Angle sum demo and hyperbolic tiling
pub struct HyperbolicCorridors {
    hyperbolic: HyperbolicSpace,
    player: GeoPoint,
    collectibles: Vec<Collectible>,
    walls: Vec<Wall>,
    collected_count: usize,
    total_collectibles: usize,
    zoom: f32,
    completed: bool,
    completion_timer: f32,
    pub runtime: LevelRuntime,
    /// Decorative impossible triangles
    deco_triangles: Vec<(f32, f32, f32)>,
    /// Size illusion demonstration pairs
    size_illusions: Vec<SizeIllusionPair>,
    /// Curvature zones for field visualization
    curvature_zones: Vec<CurvatureZone>,
}

impl HyperbolicCorridors {
    pub fn new() -> Self {
        let hyperbolic = HyperbolicSpace::new(-1.0);

        // Crystals placed at various radii to showcase conformal scaling
        let collectibles = vec![
            Collectible::new(0.0, 0.15, 'A', YELLOW),
            Collectible::new(0.15, -0.1, 'B', YELLOW),
            Collectible::new(-0.2, 0.05, 'C', YELLOW),
            Collectible::new(0.0, -0.28, 'D', YELLOW),
            Collectible::new(0.4, 0.25, 'E', YELLOW),
            Collectible::new(-0.35, -0.4, 'F', YELLOW),
            Collectible::new(-0.5, 0.25, 'G', YELLOW),
        ];

        // Ring walls with gaps — rendered as geodesics with conformal thickness
        let mut walls = Vec::new();
        let ring_radii = [0.15, 0.35, 0.55, 0.75];
        for &r in &ring_radii {
            let segments = 36;
            for i in 0..segments {
                let angle = std::f32::consts::TAU * i as f32 / segments as f32;
                let next_angle = std::f32::consts::TAU * (i + 1) as f32 / segments as f32;
                let gap_center = (angle + next_angle) / 2.0;
                let mut in_gap = false;
                for g in 0..4 {
                    let gap_angle = std::f32::consts::FRAC_PI_2 * g as f32;
                    let diff = (gap_center - gap_angle).abs();
                    let diff = diff.min(std::f32::consts::TAU - diff);
                    if diff < 0.35 { in_gap = true; break; }
                }
                if in_gap { continue; }
                walls.push(Wall::new(
                    r * angle.cos(), r * angle.sin(),
                    r * next_angle.cos(), r * next_angle.sin(),
                    Color::new(0.2, 0.4, 0.8, 1.0)));
            }
        }
        // Radial walls
        for i in 0..8 {
            let base_angle = std::f32::consts::FRAC_PI_4 * i as f32;
            let offset = 0.1;
            walls.push(Wall::new(
                0.08 * (base_angle + offset).cos(), 0.08 * (base_angle + offset).sin(),
                0.15 * (base_angle + offset).cos(), 0.15 * (base_angle + offset).sin(),
                Color::new(0.15, 0.3, 0.6, 1.0)));
            walls.push(Wall::new(
                0.25 * (base_angle - offset).cos(), 0.25 * (base_angle - offset).sin(),
                0.35 * (base_angle - offset).cos(), 0.35 * (base_angle - offset).sin(),
                Color::new(0.15, 0.3, 0.6, 1.0)));
        }

        let mut runtime = LevelRuntime::new();
        runtime.enemies.push(Enemy::new(-0.1, 0.4, 0.1, -0.4, 0.12));
        runtime.enemies.push(Enemy::new(0.3, 0.0, -0.3, 0.0, 0.10));
        runtime.enemies.push(Enemy::new(0.0, -0.15, 0.0, -0.5, 0.08));

        // Decorative impossible geometry
        let deco_triangles = vec![
            (0.0, -0.7, 0.055),
            (-0.65, 0.5, 0.045),
            (0.7, -0.45, 0.04),
        ];

        // Size illusion pairs — same-size circles at center vs edge
        let size_illusions = vec![
            SizeIllusionPair::new(0.0, 0.4, 0.7, 0.0, 0.035,
                Color::new(0.3, 0.8, 1.0, 0.8), "center vs edge"),
            SizeIllusionPair::new(-0.4, -0.2, 0.0, -0.7, 0.03,
                Color::new(1.0, 0.5, 0.3, 0.7), "near vs far"),
        ];

        // Curvature zones for field visualization
        let curvature_zones = vec![
            CurvatureZone::new(0.0, 0.0, 0.3, -1.0,
                Color::new(0.2, 0.5, 1.0, 0.4), "Negative Curvature"),
        ];

        let total = collectibles.len();
        Self {
            hyperbolic, player: GeoPoint::new(0.0, 0.0), collectibles, walls,
            collected_count: 0, total_collectibles: total, zoom: 1.5,
            completed: false, completion_timer: 0.0,
            runtime, deco_triangles, size_illusions, curvature_zones,
        }
    }
}

impl Level for HyperbolicCorridors {
    fn geometry_type(&self) -> GeometryType { GeometryType::Hyperbolic }
    fn name(&self) -> &str { "Level 1: Hyperbolic Corridors" }
    fn description(&self) -> &str {
        "Poincare disk with CONFORMAL SCALING — objects near the edge appear smaller! \
         See the curvature field, parallel line divergence, and size illusion demos. \
         Collect 7 crystals (A-G). Conformal grid thins toward the boundary."
    }
    fn objective(&self) -> &str { "Collect all 7 crystals (A-G)" }

    fn collected_fraction(&self) -> f32 {
        self.collected_count as f32 / self.total_collectibles as f32
    }

    fn reset(&mut self) { *self = Self::new(); }

    fn update(&mut self, dt: f32, move_x: f32, move_y: f32, _action: bool, dash: bool) -> LevelResult {
        if dash { let _ = self.runtime.dash.try_dash(); }
        self.runtime.update(dt, self.player.x, self.player.y);

        if self.completed {
            self.completion_timer += dt;
            if self.completion_timer > 2.0 {
                return LevelResult::Completed(self.collected_count as u32 * 100 + self.runtime.combo.total_bonus);
            }
            return LevelResult::Playing;
        }

        if self.runtime.check_enemy_collision(self.player) {
            self.runtime.effects.shake(8.0, 0.3);
            self.runtime.effects.flash(Color::new(1.0, 0.0, 0.0, 1.0), 0.2);
            self.runtime.invuln_timer = 1.5;
            let r = (self.player.x * self.player.x + self.player.y * self.player.y).sqrt();
            if r > 0.01 {
                self.player.x -= self.player.x / r * 0.05;
                self.player.y -= self.player.y / r * 0.05;
            }
            self.runtime.combo.combo = 0;
            self.runtime.particles.emit_burst(self.player.x, self.player.y, RED, 20);
        }

        let speed = 0.003 * self.runtime.dash.speed_multiplier();
        self.player = self.hyperbolic.move_point(self.player, move_x, move_y, speed);

        for c in &mut self.collectibles { c.update(dt); }

        let collect_dist = 0.035;
        for c in &mut self.collectibles {
            if !c.collected {
                let dist = self.hyperbolic.distance(self.player, c.pos);
                if dist < collect_dist {
                    c.collected = true;
                    self.collected_count += 1;
                    self.runtime.combo.on_collect(self.runtime.level_time, 100);
                    self.runtime.particles.emit_burst(c.pos.x, c.pos.y, c.color, 15);
                    self.runtime.effects.shake(3.0, 0.15);
                    self.runtime.effects.flash(Color::new(1.0, 1.0, 0.3, 1.0), 0.1);
                }
            }
        }

        if self.collected_count >= self.total_collectibles && !self.completed {
            self.completed = true;
            self.completion_timer = 0.0;
            self.runtime.effects.flash(YELLOW, 0.5);
            self.runtime.effects.shake(5.0, 0.5);
        }

        let r = (self.player.x * self.player.x + self.player.y * self.player.y).sqrt();
        self.zoom = 1.5 + r * 1.5;

        LevelResult::Playing
    }

    fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        let cx = self.player.x;
        let cy = self.player.y;
        let t = self.runtime.level_time;

        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));

        // Boundary circle (always Euclidean — it IS a Euclidean circle)
        renderer::draw_circle_outline_world(0.0, 0.0, 1.0, cx, cy, self.zoom, sw, sh,
            Color::new(0.4, 0.1, 0.1, 1.0), 3.0);

        // === CONFORMAL GEODESIC GRID ===
        // Grid rings use conformal scaling — they get thinner near the edge
        let grid_color = Color::new(0.08, 0.12, 0.25, 0.6);
        for ring in 1..=12 {
            let r = ring as f32 * 0.08;
            if r < 0.99 {
                illusion::draw_geodesic_ring_conformal(0.0, 0.0, r, 24,
                    cx, cy, self.zoom, sw, sh, grid_color, 1.0, 8);
            }
        }
        // Radial lines with conformal thickness
        for i in 0..8 {
            let angle = std::f32::consts::TAU * i as f32 / 8.0;
            illusion::draw_geodesic_radial_conformal(0.0, 0.0, angle, 0.01, 0.99,
                cx, cy, self.zoom, sw, sh, grid_color, 1.0);
        }

        // === CURVATURE FIELD VISUALIZATION ===
        // Shows arrows diverging outward — the signature of negative curvature
        for zone in &self.curvature_zones {
            illusion::draw_curvature_field(
                zone.x, zone.y, zone.radius, zone.curvature,
                cx, cy, self.zoom, sw, sh, zone.color, t);

            // Label for the curvature zone
            renderer::draw_text_world(&zone.label, zone.x - 0.04, zone.y + zone.radius + 0.02,
                cx, cy, self.zoom, sw, sh, 0.02,
                Color::new(zone.color.r, zone.color.g, zone.color.b, 0.5 + t.sin() * 0.15));
        }

        // === PARALLEL LINE DEVIATION DEMO ===
        // Shows how parallel lines diverge in hyperbolic space
        illusion::draw_parallel_deviation(
            0.3, 0.5,  // start position
            -std::f32::consts::FRAC_PI_2,  // direction: downward
            0.06,      // spacing
            0.25,      // length
            -0.8,      // negative curvature = diverge
            cx, cy, self.zoom, sw, sh,
            Color::new(0.3, 0.7, 1.0, 0.7), t);

        // === WALLS AS CONFORMAL GEODESIC ARCS ===
        for wall in &self.walls {
            illusion::draw_geodesic_line_conformal(
                wall.p1.x, wall.p1.y, wall.p2.x, wall.p2.y,
                cx, cy, self.zoom, sw, sh, wall.color, 3.0, 8);
        }

        // === ANGLE SUM DEMO ===
        let demo_a = GeoPoint::new(-0.5, -0.5);
        let demo_b = GeoPoint::new(0.5, -0.5);
        let demo_c = GeoPoint::new(0.0, 0.4);
        let angle_sum = self.hyperbolic.triangle_angle_sum(demo_a, demo_b, demo_c);
        illusion::draw_angle_sum_demo(
            demo_a.x, demo_a.y, demo_b.x, demo_b.y, demo_c.x, demo_c.y,
            angle_sum, 180.0,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.15, 0.3, 0.6, 0.25));

        // === HYPERBOLIC TILING HINT ===
        illusion::draw_hyperbolic_tiling_hint(
            0.0, 0.0, 0.15, 1,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.1, 0.2, 0.5, 0.15), t);

        // === SIZE ILLUSION PAIRS ===
        // Same-size objects at different distances appear different sizes!
        for pair in &self.size_illusions {
            renderer::draw_size_illusion_pair(
                pair.pos_a.x, pair.pos_a.y, pair.pos_b.x, pair.pos_b.y,
                pair.radius,
                cx, cy, self.zoom, sw, sh,
                pair.color, "A", "B",
                true,  // use conformal scaling
            );
        }

        // === DECORATIVE IMPOSSIBLE GEOMETRY ===
        for &(dx, dy, ds) in &self.deco_triangles {
            illusion::draw_impossible_triangle(
                dx, dy, ds, cx, cy, self.zoom, sw, sh,
                Color::new(0.3, 0.5, 0.9, 0.4), t);
        }

        // Enemies (drawn with conformal scaling!)
        for e in &self.runtime.enemies {
            // Use conformal scaling for enemies near the edge
            let r2 = e.pos.x * e.pos.x + e.pos.y * e.pos.y;
            let conformal = if r2 < 1.0 { (1.0 - r2) / 2.0 } else { 0.01 };
            let _effective_zoom = self.zoom * (conformal * 4.0).max(0.15);

            // Alert radius
            renderer::draw_circle_world_conformal(
                e.pos.x, e.pos.y, e.alert_radius,
                cx, cy, self.zoom, sw, sh,
                Color::new(1.0, 0.0, 0.0, 0.06));
            // Glow
            let pulse = (e.phase).sin() * 0.005;
            renderer::draw_circle_world_conformal(
                e.pos.x, e.pos.y, e.radius + 0.015 + pulse,
                cx, cy, self.zoom, sw, sh,
                Color::new(1.0, 0.1, 0.0, 0.15));
            // Body
            renderer::draw_circle_world_conformal(
                e.pos.x, e.pos.y, e.radius + pulse,
                cx, cy, self.zoom, sw, sh, e.color);
            // Eye
            renderer::draw_circle_world_conformal(
                e.pos.x, e.pos.y, e.radius * 0.35,
                cx, cy, self.zoom, sw, sh,
                Color::new(1.0, 1.0, 0.5, 0.9));
        }

        // Collectibles with CONFORMAL SCALING — the key visual!
        // Objects near the edge appear much smaller, creating the illusion
        // that the disk contains far more space than it seems
        for c in &self.collectibles {
            if !c.collected {
                let er = c.effective_radius();
                let by = c.bob_y();

                // Glow (conformal)
                renderer::draw_circle_world_conformal(
                    c.pos.x, c.pos.y + by, er * 2.0,
                    cx, cy, self.zoom, sw, sh,
                    Color::new(c.color.r, c.color.g, c.color.b, 0.2));
                // Body (conformal)
                renderer::draw_circle_world_conformal(
                    c.pos.x, c.pos.y + by, er,
                    cx, cy, self.zoom, sw, sh, c.color);
                // Label (conformal)
                renderer::draw_text_world_conformal(
                    &c.label.to_string(),
                    c.pos.x - 0.01, c.pos.y + by - 0.01,
                    cx, cy, self.zoom, sw, sh, 0.04, WHITE);
            }
        }

        // Trail
        renderer::draw_trail(&self.runtime.dash.trail, cx, cy, self.zoom, sw, sh, GREEN);

        // Player
        let player_r = 0.025;
        let invuln_flash = if self.runtime.invuln_timer > 0.0 {
            (self.runtime.invuln_timer * 10.0).sin().abs() > 0.3
        } else { false };
        if !invuln_flash {
            renderer::draw_circle_world(self.player.x, self.player.y, player_r * 1.8,
                cx, cy, self.zoom, sw, sh, Color::new(0.0, 1.0, 0.5, 0.2));
            renderer::draw_circle_world(self.player.x, self.player.y, player_r,
                cx, cy, self.zoom, sw, sh, GREEN);
        }

        // Particles
        self.runtime.particles.draw(cx, cy, self.zoom, sw, sh);

        // Proximity indicators
        let half_w = sw * 0.45 * self.zoom;
        let half_h = sh * 0.45 * self.zoom;
        for c in &self.collectibles {
            if !c.collected {
                let sx = sw / 2.0 + (c.pos.x - cx) * half_w;
                let sy = sh / 2.0 + (c.pos.y - cy) * half_h;
                renderer::draw_proximity_indicator(sw / 2.0, sh / 2.0, sx, sy, sw, sh, YELLOW, &c.label.to_string());
            }
        }

        // Poincaré disk boundary with animated glow
        renderer::draw_poincare_boundary(cx, cy, self.zoom, sw, sh,
            Color::new(0.0, 0.4, 0.6, 0.6), t);

        // Ideal points on boundary
        renderer::draw_ideal_points(8, cx, cy, self.zoom, sw, sh,
            Color::new(0.0, 0.8, 1.0, 0.6), t);

        // Horocycle demonstration
        for i in 0..4 {
            let angle = std::f32::consts::FRAC_PI_2 * i as f32 + 0.3;
            renderer::draw_horocycle(angle, 0.25 + i as f32 * 0.05,
                cx, cy, self.zoom, sw, sh,
                Color::new(0.2, 0.6, 0.8, 0.2 + i as f32 * 0.05),
                1.0, t);
        }

        // Klein model geodesic comparison (faint straight lines)
        for i in 0..6 {
            let angle = std::f32::consts::TAU * i as f32 / 6.0 + t * 0.02;
            renderer::draw_klein_geodesic(
                -0.7 * angle.cos(), -0.7 * angle.sin(),
                0.7 * angle.cos(), 0.7 * angle.sin(),
                cx, cy, self.zoom, sw, sh,
                Color::new(0.4, 0.2, 0.2, 0.08), 0.5);
        }

        // Engine feature labels
        renderer::draw_text_world("CONFORMAL SCALING: objects shrink near edge", 0.0, 0.88,
            cx, cy, self.zoom, sw, sh, 0.02,
            Color::new(0.3, 0.5, 0.8, 0.4 + (t * 0.5).sin() * 0.1));
        renderer::draw_text_world("Geodesics = circular arcs", 0.0, 0.93,
            cx, cy, self.zoom, sw, sh, 0.018,
            Color::new(0.25, 0.4, 0.7, 0.35));

        if self.completed {
            let flash = (self.completion_timer * 5.0).sin().abs();
            renderer::draw_circle_world(self.player.x, self.player.y, 0.1 + flash * 0.05,
                cx, cy, self.zoom, sw, sh, Color::new(1.0, 1.0, 0.0, 0.3));
        }

        self.runtime.effects.draw_flash();
    }
}