use macroquad::prelude::*;
use crate::geometry::{GeoPoint, GeometryType, spherical::SphericalSpace};
use crate::engine::game::{Level, LevelResult, LevelRuntime, Collectible, Wall, Enemy};
use crate::engine::renderer;
use crate::engine::illusion;

/// Level 2: Spherical Labyrinth
/// Walk on a sphere with great circle geodesics and spherical geometry illusions.
///
/// ENHANCED with engine features:
/// - Great circle geodesic rendering for walls (curved on the Mercator projection)
/// - Angle excess demo: triangle angle sum > 180° on a sphere
/// - Parallel line convergence: lines converge toward the poles
/// - Spherical conformal scaling for objects (compressed horizontally near poles)
/// - Curvature field showing converging space
pub struct SphericalLabyrinth {
    spherical: SphericalSpace,
    player: GeoPoint,
    collectibles: Vec<Collectible>,
    walls: Vec<Wall>,
    collected_count: usize,
    total_collectibles: usize,
    zoom: f32,
    wrap_flash: f32,
    wrap_count: u32,
    completed: bool,
    completion_timer: f32,
    pub runtime: LevelRuntime,
}

impl SphericalLabyrinth {
    pub fn new() -> Self {
        let spherical = SphericalSpace::new(1.0);
        let collectibles = vec![
            Collectible::new(0.5, 0.3, '1', ORANGE),
            Collectible::new(-0.5, 0.3, '2', ORANGE),
            Collectible::new(0.0, 0.6, '3', ORANGE),
            Collectible::new(0.7, -0.2, '4', ORANGE),
            Collectible::new(-0.7, -0.2, '5', ORANGE),
            Collectible::new(0.3, -0.5, '6', ORANGE),
        ];

        // Walls — stored as world-space segments, rendered as great circles
        let mut walls = Vec::new();
        let lat_bands = [-0.3, 0.0, 0.3];
        for &lat in &lat_bands {
            let gap_centers = [-0.4, 0.4];
            for x in -10..=10 {
                let wx = x as f32 * 0.1;
                let mut in_gap = false;
                for &gc in &gap_centers { if (wx - gc).abs() < 0.2 { in_gap = true; break; } }
                if in_gap || x == -10 { continue; }
                walls.push(Wall::new(wx - 0.1, lat, wx, lat, Color::new(0.6, 0.2, 0.2, 1.0)));
            }
        }
        let lon_positions = [-0.5, 0.0, 0.5];
        for &lon in &lon_positions {
            for y in -8..=8 {
                let wy = y as f32 * 0.1;
                if wy.abs() > 0.85 || y == -8 { continue; }
                let mut in_gap = false;
                for &gl in &[-0.15, 0.15, -0.45, 0.45] { if (wy - gl).abs() < 0.1 { in_gap = true; break; } }
                if in_gap { continue; }
                walls.push(Wall::new(lon, wy - 0.1, lon, wy, Color::new(0.5, 0.15, 0.15, 1.0)));
            }
        }

        let mut runtime = LevelRuntime::new();
        runtime.enemies.push(Enemy::new(0.0, -0.6, 0.0, -0.2, 0.15));
        runtime.enemies.push(Enemy::new(0.6, 0.0, 0.6, 0.5, 0.10));
        runtime.enemies.push(Enemy::new(-0.3, 0.0, 0.3, 0.0, 0.12));

        let total = collectibles.len();
        Self {
            spherical, player: GeoPoint::new(0.0, 0.0), collectibles, walls,
            collected_count: 0, total_collectibles: total, zoom: 2.0,
            wrap_flash: 0.0, wrap_count: 0, completed: false, completion_timer: 0.0,
            runtime,
        }
    }
}

impl Level for SphericalLabyrinth {
    fn geometry_type(&self) -> GeometryType { GeometryType::Spherical }
    fn name(&self) -> &str { "Level 2: Spherical Labyrinth" }
    fn description(&self) -> &str {
        "Walk on a sphere with GREAT CIRCLE geodesics and POSITIVE CURVATURE. \
         Lines CONVERGE toward poles. Triangle angle sum > 180 deg. \
         Collect 6 orbs (1-6). Walls follow great circles!"
    }
    fn objective(&self) -> &str { "Collect all 6 orbs (1-6)" }

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
                return LevelResult::Completed(self.collected_count as u32 * 150 + self.runtime.combo.total_bonus - self.wrap_count);
            }
            return LevelResult::Playing;
        }

        if self.runtime.check_enemy_collision(self.player) {
            self.runtime.effects.shake(8.0, 0.3);
            self.runtime.effects.flash(Color::new(1.0, 0.0, 0.0, 1.0), 0.2);
            self.runtime.invuln_timer = 1.5;
            self.runtime.combo.combo = 0;
            self.runtime.particles.emit_burst(self.player.x, self.player.y, RED, 20);
        }

        let old_pos = self.player;
        let speed = 1.5 * self.runtime.dash.speed_multiplier();
        self.player = self.spherical.move_point(self.player, move_x, move_y, speed);

        if self.spherical.did_wrap(old_pos, self.player) {
            self.wrap_flash = 1.0;
            self.wrap_count += 1;
            self.runtime.effects.chromatic(3.0, 0.4);
            self.runtime.particles.emit_burst(self.player.x, self.player.y, Color::new(0.8, 0.1, 0.1, 1.0), 12);
        }
        self.wrap_flash = (self.wrap_flash - dt * 2.0).max(0.0);

        for c in &mut self.collectibles { c.update(dt); }
        for c in &mut self.collectibles {
            if !c.collected {
                let dist = self.spherical.distance(self.player, c.pos);
                if dist < 0.15 {
                    c.collected = true;
                    self.collected_count += 1;
                    self.runtime.combo.on_collect(self.runtime.level_time, 150);
                    self.runtime.particles.emit_burst(c.pos.x, c.pos.y, c.color, 15);
                    self.runtime.effects.shake(3.0, 0.15);
                }
            }
        }

        if self.collected_count >= self.total_collectibles && !self.completed {
            self.completed = true;
            self.completion_timer = 0.0;
            self.runtime.effects.flash(ORANGE, 0.5);
        }
        LevelResult::Playing
    }

    fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        let cx = self.player.x;
        let cy = self.player.y;
        let t = self.runtime.level_time;

        let bg_r = 0.05 + self.wrap_flash * 0.1;
        clear_background(Color::new(bg_r, 0.02, 0.05, 1.0));

        // === GRID WITH GREAT CIRCLE CURVATURE ===
        // Latitude lines — these are great circles, so they curve on the Mercator projection
        let grid_color = Color::new(0.15, 0.05, 0.1, 0.4);
        for i in -8..=8 {
            let y = i as f32 * 0.1;
            // Horizontal lines are straight in Mercator (latitude lines)
            renderer::draw_line_world(-1.2, y, 1.2, y, cx, cy, self.zoom, sw, sh, grid_color, 1.0);
        }
        // Longitude lines — these are ALSO great circles, and DO curve on Mercator
        for i in -12..=12 {
            let x = i as f32 * 0.1;
            // Draw longitude as great circle arcs (they curve!)
            illusion::draw_great_circle_line(
                x, -0.95, x, 0.95,
                cx, cy, self.zoom, sw, sh,
                Color::new(0.12, 0.05, 0.1, 0.5), 1.0, 16);
        }

        // Edge indicators with wrap hint
        let edge_color = Color::new(0.8, 0.1, 0.1, 0.5 + self.wrap_flash * 0.5);
        renderer::draw_line_world(-1.0, -1.5, -1.0, 1.5, cx, cy, self.zoom, sw, sh, edge_color, 2.0);
        renderer::draw_line_world(1.0, -1.5, 1.0, 1.5, cx, cy, self.zoom, sw, sh, edge_color, 2.0);
        // Wrap arrows
        renderer::draw_text_world("<< WRAPS >>", -1.0, 0.92, cx, cy, self.zoom, sw, sh, 0.03,
            Color::new(0.8, 0.3, 0.3, 0.4 + self.wrap_flash * 0.4));
        renderer::draw_text_world("<< WRAPS >>", 0.95, 0.92, cx, cy, self.zoom, sw, sh, 0.03,
            Color::new(0.8, 0.3, 0.3, 0.4 + self.wrap_flash * 0.4));

        // Poles
        let pole_color = Color::new(1.0, 0.3, 0.3, 0.6);
        renderer::draw_text_world("N POLE (all longitudes meet)", 0.0, -0.92, cx, cy, self.zoom, sw, sh, 0.04, pole_color);
        renderer::draw_text_world("S POLE (all longitudes meet)", 0.0, 0.95, cx, cy, self.zoom, sw, sh, 0.04, pole_color);

        // === CURVATURE FIELD: POSITIVE CURVATURE ===
        // Shows arrows converging — the signature of positive (spherical) curvature
        illusion::draw_curvature_field(
            0.0, 0.0, 0.5, 1.0,  // positive curvature = converge
            cx, cy, self.zoom, sw, sh,
            Color::new(1.0, 0.3, 0.3, 0.35), t);

        // === PARALLEL LINE CONVERGENCE DEMO ===
        // On a sphere, parallel lines converge and eventually meet (at the poles!)
        illusion::draw_parallel_deviation(
            -0.6, -0.7,
            std::f32::consts::FRAC_PI_2,  // direction: rightward
            0.05, 0.3, 0.8,  // positive curvature = converge
            cx, cy, self.zoom, sw, sh,
            Color::new(1.0, 0.5, 0.3, 0.7), t);

        // === ANGLE EXCESS DEMO ===
        // Spherical triangle: angle sum > 180°
        let tri_a = GeoPoint::new(-0.3, -0.5);
        let tri_b = GeoPoint::new(0.3, -0.5);
        let tri_c = GeoPoint::new(0.0, 0.3);
        let angle_sum = self.spherical.triangle_angle_sum(tri_a, tri_b, tri_c);
        // Draw edges as great circles
        illusion::draw_great_circle_line(tri_a.x, tri_a.y, tri_b.x, tri_b.y,
            cx, cy, self.zoom, sw, sh, Color::new(0.8, 0.3, 0.1, 0.5), 1.5, 12);
        illusion::draw_great_circle_line(tri_b.x, tri_b.y, tri_c.x, tri_c.y,
            cx, cy, self.zoom, sw, sh, Color::new(0.8, 0.3, 0.1, 0.5), 1.5, 12);
        illusion::draw_great_circle_line(tri_c.x, tri_c.y, tri_a.x, tri_a.y,
            cx, cy, self.zoom, sw, sh, Color::new(0.8, 0.3, 0.1, 0.5), 1.5, 12);

        let mid_x = (tri_a.x + tri_b.x + tri_c.x) / 3.0;
        let mid_y = (tri_a.y + tri_b.y + tri_c.y) / 3.0;
        renderer::draw_text_world(
            &format!("{:.0} deg (>180!)", angle_sum),
            mid_x - 0.04, mid_y - 0.01,
            cx, cy, self.zoom, sw, sh, 0.03,
            Color::new(1.0, 0.5, 0.2, 0.7));

        // === WALLS AS GREAT CIRCLE ARCS ===
        for wall in &self.walls {
            // Use great circle rendering for walls on the sphere
            illusion::draw_great_circle_line(
                wall.p1.x, wall.p1.y, wall.p2.x, wall.p2.y,
                cx, cy, self.zoom, sw, sh,
                wall.color, 3.0, 8);
        }

        // Enemies
        for e in &self.runtime.enemies {
            renderer::draw_enemy(e, cx, cy, self.zoom, sw, sh);
        }

        // Collectibles with spherical conformal scaling
        for c in &self.collectibles {
            if !c.collected {
                let er = c.effective_radius();
                let by = c.bob_y();
                renderer::draw_circle_world_spherical(
                    c.pos.x, c.pos.y + by, er * 2.0,
                    cx, cy, self.zoom, sw, sh,
                    Color::new(c.color.r, c.color.g, c.color.b, 0.2));
                renderer::draw_circle_world_spherical(
                    c.pos.x, c.pos.y + by, er,
                    cx, cy, self.zoom, sw, sh, c.color);
                renderer::draw_text_world(
                    &c.label.to_string(), c.pos.x - 0.01, c.pos.y + by - 0.01,
                    cx, cy, self.zoom, sw, sh, 0.04, WHITE);
            }
        }

        // Trail
        renderer::draw_trail(&self.runtime.dash.trail, cx, cy, self.zoom, sw, sh, BLUE);

        // Player
        let player_r = 0.025;
        let invuln_flash = if self.runtime.invuln_timer > 0.0 {
            (self.runtime.invuln_timer * 10.0).sin().abs() > 0.3
        } else { false };
        if !invuln_flash {
            renderer::draw_circle_world(self.player.x, self.player.y, player_r * 1.8,
                cx, cy, self.zoom, sw, sh, Color::new(0.0, 0.5, 1.0, 0.2));
            renderer::draw_circle_world(self.player.x, self.player.y, player_r,
                cx, cy, self.zoom, sw, sh, BLUE);
        }

        // Scale indicator
        let scale_factor = self.spherical.scale_at(self.player);
        renderer::draw_text_world(&format!("Mercator scale: {:.2}x (cos lat)", scale_factor),
            self.player.x, self.player.y + 0.05, cx, cy, self.zoom, sw, sh, 0.025,
            Color::new(0.5, 0.5, 0.5, 0.6));

        // Particles
        self.runtime.particles.draw(cx, cy, self.zoom, sw, sh);

        // Proximity
        let half_w = sw * 0.45 * self.zoom;
        let half_h = sh * 0.45 * self.zoom;
        for c in &self.collectibles {
            if !c.collected {
                let sx = sw / 2.0 + (c.pos.x - cx) * half_w;
                let sy = sh / 2.0 + (c.pos.y - cy) * half_h;
                renderer::draw_proximity_indicator(sw / 2.0, sh / 2.0, sx, sy, sw, sh, ORANGE, &c.label.to_string());
            }
        }

        // Engine feature labels
        renderer::draw_text_world("GREAT CIRCLE geodesics (curved on Mercator)", 0.0, -0.98,
            cx, cy, self.zoom, sw, sh, 0.018,
            Color::new(0.7, 0.3, 0.2, 0.4 + (t * 0.5).sin() * 0.1));

        if self.completed {
            let flash = (self.completion_timer * 5.0).sin().abs();
            renderer::draw_circle_world(self.player.x, self.player.y, 0.1 + flash * 0.05,
                cx, cy, self.zoom, sw, sh, Color::new(1.0, 0.5, 0.0, 0.3));
        }

        self.runtime.effects.draw_flash();
    }
}