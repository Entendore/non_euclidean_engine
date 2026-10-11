use macroquad::prelude::*;
use crate::geometry::{GeoPoint, GeometryType};
use crate::engine::game::{Level, LevelResult, LevelRuntime, Collectible, Wall, Portal, PortalType};
use crate::engine::renderer;
use crate::engine::illusion;

/// Level 3: The Warp
/// Geometry transitions between Hyperbolic, Euclidean, and Spherical.
///
/// FULLY REBUILT with engine features:
/// - 6 portal types: Normal(N), Mirror(M), OneWay(A>B/B>A), GeometryShift(G), AutoWarp(W), Recursive(R)
/// - Recursive portal rendering (portal within portal infinite regression)
/// - Wormhole corridor visualization between portal pairs
/// - Geodesic link arcs connecting portal endpoints
/// - Gravity lens effect around portals
/// - Auto-teleport portals (walk through, no action key)
/// - Portal chaining (enter one, teleport to linked portal)
/// - Curvature field in each geometry zone
/// - Impossible geometry decorations
/// - Parallel line demos for each geometry type
pub struct TheWarp {
    player: GeoPoint,
    collectibles: Vec<Collectible>,
    walls: Vec<Wall>,
    portals: Vec<Portal>,
    collected_count: usize,
    total_collectibles: usize,
    zoom: f32,
    geometry_blend: f32,
    completed: bool,
    completion_timer: f32,
    portal_cooldown: f32,
    zones: Vec<(f32, f32)>,
    pub runtime: LevelRuntime,
    portal_illusion: illusion::PortalIllusion,
    deco_tesseracts: Vec<(f32, f32, f32)>,
    deco_triangles: Vec<(f32, f32, f32)>,
    deco_staircases: Vec<(f32, f32, f32)>,
}

impl TheWarp {
    pub fn new() -> Self {
        let collectibles = vec![
            Collectible::new(-1.5, 0.2, 'H', Color::new(0.0, 0.9, 0.9, 1.0)),
            Collectible::new(-1.8, -0.3, 'Y', Color::new(0.0, 0.9, 0.9, 1.0)),
            Collectible::new(0.0, 0.4, 'E', GREEN),
            Collectible::new(0.3, -0.3, 'U', GREEN),
            Collectible::new(-0.3, -0.5, 'C', GREEN),
            Collectible::new(1.5, 0.2, 'S', RED),
            Collectible::new(1.8, -0.3, 'P', RED),
            Collectible::new(1.3, -0.5, 'W', RED),
        ];
        let zones = vec![(-3.0, 0.0), (-1.0, 0.1), (-0.3, 0.5), (0.3, 0.5), (1.0, 0.9), (3.0, 1.0)];
        let mut walls = Vec::new();
        for i in 0..20 {
            let y = -1.0 + i as f32 * 0.1;
            if (i > 4 && i < 8) || (i > 12 && i < 16) { continue; }
            walls.push(Wall::new(-1.0, y, -1.0, y + 0.1, Color::new(0.1, 0.3, 0.8, 0.6)));
        }
        for i in 0..20 {
            let y = -1.0 + i as f32 * 0.1;
            if (i > 3 && i < 7) || (i > 11 && i < 15) { continue; }
            walls.push(Wall::new(1.0, y, 1.0, y + 0.1, Color::new(0.8, 0.2, 0.2, 0.6)));
        }
        walls.push(Wall::new(-2.5, -0.8, -1.0, -0.8, Color::new(0.3, 0.3, 0.3, 0.5)));
        walls.push(Wall::new(-2.5, 0.8, -1.0, 0.8, Color::new(0.3, 0.3, 0.3, 0.5)));
        walls.push(Wall::new(-0.3, -0.7, 0.3, -0.7, Color::new(0.3, 0.3, 0.3, 0.5)));
        walls.push(Wall::new(-0.3, 0.7, 0.3, 0.7, Color::new(0.3, 0.3, 0.3, 0.5)));
        walls.push(Wall::new(1.0, -0.8, 2.5, -0.8, Color::new(0.3, 0.3, 0.3, 0.5)));
        walls.push(Wall::new(1.0, 0.8, 2.5, 0.8, Color::new(0.3, 0.3, 0.3, 0.5)));

        // === 6 PORTAL TYPES showcasing all engine portal features ===
        let portals = vec![
            // Portal 0: Normal bidirectional — classic portal with geodesic link
            {
                let mut p = Portal::with_type(-2.0, 0.0, 2.0, 0.0, PortalType::Normal);
                p.show_geodesic_link = true;
                p.show_window = true;
                p
            },
            // Portal 1: Mirror portal — flips X, non-orientable space
            Portal::with_type(-0.5, 0.0, 0.5, 0.0, PortalType::Mirror),
            // Portal 2: One-way (A→B only)
            {
                let mut p = Portal::with_type(-2.2, -0.5, 2.2, 0.5, PortalType::OneWayAB);
                p.radius = 0.06;
                p.show_window = true;
                p.show_geodesic_link = true;
                p
            },
            // Portal 3: Geometry shift
            {
                let mut p = Portal::with_type(0.0, 0.6, 0.0, -0.6, PortalType::GeometryShift);
                p.radius = 0.05;
                p.show_window = true;
                p.show_geodesic_link = true;
                p
            },
            // Portal 4: Auto-warp — walk through it, no action key!
            // Has wormhole corridor visualization
            {
                let mut p = Portal::with_type(-1.5, -0.65, 1.5, -0.65, PortalType::AutoWarp);
                p.radius = 0.04;
                p.show_geodesic_link = true;
                p.show_corridor = true;
                p
            },
            // Portal 5: Recursive portal — portal within portal infinite regression
            {
                let mut p = Portal::with_type(-0.7, 0.55, 0.7, 0.55, PortalType::Recursive);
                p.radius = 0.045;
                p.show_geodesic_link = true;
                p
            },
        ];

        let mut runtime = LevelRuntime::new();
        runtime.enemies.push(crate::engine::game::Enemy::new(-1.5, 0.5, -1.5, -0.3, 0.08));
        runtime.enemies.push(crate::engine::game::Enemy::new(0.0, 0.5, 0.0, -0.5, 0.10));
        runtime.enemies.push(crate::engine::game::Enemy::new(1.5, 0.4, 1.5, -0.4, 0.08));

        let deco_tesseracts = vec![
            (-2.0, 0.7, 0.08),
            (2.0, 0.7, 0.08),
            (0.0, 0.85, 0.06),
        ];
        let deco_triangles = vec![
            (-2.5, -0.5, 0.05),
            (2.5, -0.5, 0.05),
            (0.4, 0.65, 0.04),
            (-0.4, 0.65, 0.04),
        ];
        let deco_staircases = vec![
            (-1.5, -0.7, 0.07),
            (1.5, -0.7, 0.07),
        ];

        let total = collectibles.len();
        Self {
            player: GeoPoint::new(0.0, 0.0), collectibles, walls, portals,
            collected_count: 0, total_collectibles: total, zoom: 1.5,
            geometry_blend: 0.5, completed: false, completion_timer: 0.0,
            portal_cooldown: 0.0, zones, runtime,
            portal_illusion: illusion::PortalIllusion::new(),
            deco_tesseracts, deco_triangles, deco_staircases,
        }
    }

    fn get_blend_at(&self, x: f32) -> f32 {
        for i in 0..self.zones.len() - 1 {
            let (x0, b0) = self.zones[i];
            let (x1, b1) = self.zones[i + 1];
            if x >= x0 && x <= x1 {
                let t = (x - x0) / (x1 - x0);
                return b0 + (b1 - b0) * t;
            }
        }
        if x < self.zones[0].0 { return self.zones[0].1; }
        return self.zones.last().unwrap().1;
    }

    fn blend_color(&self) -> Color {
        let b = self.geometry_blend;
        let r = if b > 0.5 { (b - 0.5) * 2.0 } else { 0.0 };
        let g = if b <= 0.5 { 1.0 - b * 2.0 } else { 1.0 - (b - 0.5) * 2.0 };
        let bl = if b < 0.5 { 1.0 - b * 2.0 } else { 0.0 };
        Color::new(r * 0.15, g * 0.08, bl * 0.15 + 0.03, 1.0)
    }

    fn collectibles_for_window(&self) -> Vec<(f32, f32, bool, char)> {
        self.collectibles.iter().map(|c| (c.pos.x, c.pos.y, c.collected, c.label)).collect()
    }

    /// Handle teleportation for a portal (shared logic for action and auto).
    fn try_teleport(&mut self, portal_idx: usize, from_a: bool) -> bool {
        let portal = &self.portals[portal_idx];
        let (enter_pos, exit_pos) = if from_a {
            (portal.pos_a, portal.pos_b)
        } else {
            (portal.pos_b, portal.pos_a)
        };
        let dist = self.player.distance_to(enter_pos);
        if dist < portal.radius + 0.08 {
            // Check direction permission
            if from_a && !portal.can_enter_from_a() { return false; }
            if !from_a && !portal.can_enter_from_b() { return false; }

            self.player = exit_pos;

            // Mirror flip
            if portal.mirror_flip_x {
                self.player.x = 2.0 * exit_pos.x - self.player.x;
            }

            self.portal_cooldown = 1.0;

            // Effects
            let entry_color = if from_a { portal.color_a } else { portal.color_b };
            let exit_color = if from_a { portal.color_b } else { portal.color_a };
            let entry_pos_f = if from_a { portal.pos_a } else { portal.pos_b };
            let exit_pos_f = if from_a { portal.pos_b } else { portal.pos_a };

            self.runtime.particles.emit_burst(entry_pos_f.x, entry_pos_f.y, entry_color, 20);
            self.runtime.particles.emit_burst(exit_pos_f.x, exit_pos_f.y, exit_color, 20);
            self.runtime.effects.flash(Color::new(0.5, 0.0, 1.0, 1.0), 0.2);
            self.runtime.effects.chromatic(4.0, 0.5);
            self.runtime.effects.shake(4.0, 0.2);

            if portal.portal_type == PortalType::GeometryShift {
                self.runtime.effects.flash(Color::new(0.0, 1.0, 0.5, 1.0), 0.3);
                self.runtime.effects.shake(6.0, 0.4);
            }
            if portal.portal_type == PortalType::Recursive {
                self.runtime.effects.flash(Color::new(0.8, 0.0, 1.0, 1.0), 0.25);
                self.runtime.effects.shake(5.0, 0.3);
            }

            return true;
        }
        false
    }
}

impl Level for TheWarp {
    fn geometry_type(&self) -> GeometryType { GeometryType::Euclidean }
    fn name(&self) -> &str { "Level 3: The Warp" }
    fn description(&self) -> &str {
        "6 portal types! N=Normal, M=Mirror, A>B=OneWay, G=GeometryShift, W=AutoWarp, R=Recursive. \
         Wormhole corridors, recursive portals, gravity lenses, geodesic links. Collect 8 gems!"
    }
    fn objective(&self) -> &str { "Collect all 8 gems (H,Y,E,U,C,S,P,W)" }

    fn collected_fraction(&self) -> f32 {
        self.collected_count as f32 / self.total_collectibles as f32
    }

    fn reset(&mut self) { *self = Self::new(); }

    fn update(&mut self, dt: f32, move_x: f32, move_y: f32, action: bool, dash: bool) -> LevelResult {
        if dash { let _ = self.runtime.dash.try_dash(); }
        self.runtime.update(dt, self.player.x, self.player.y);
        self.portal_illusion.update(dt);

        if self.completed {
            self.completion_timer += dt;
            if self.completion_timer > 2.5 {
                return LevelResult::Completed(self.collected_count as u32 * 200 + self.runtime.combo.total_bonus);
            }
            return LevelResult::Playing;
        }

        if self.portal_cooldown > 0.0 { self.portal_cooldown -= dt; }

        // Enemy collision
        if self.runtime.check_enemy_collision(self.player) {
            self.runtime.effects.shake(8.0, 0.3);
            self.runtime.effects.flash(Color::new(1.0, 0.0, 0.0, 1.0), 0.2);
            self.runtime.invuln_timer = 1.5;
            self.runtime.combo.combo = 0;
            self.runtime.particles.emit_burst(self.player.x, self.player.y, RED, 20);
        }

        self.geometry_blend = self.get_blend_at(self.player.x);
        let hyperbolic_factor = 1.0 - self.geometry_blend;
        let spherical_factor = self.geometry_blend;
        let speed = 0.004 * (1.0 + hyperbolic_factor * 0.5 - spherical_factor * 0.2) * self.runtime.dash.speed_multiplier();

        let mut dx = move_x * speed;
        let dy = move_y * speed;
        if self.geometry_blend < 0.3 {
            dx += (-2.0 - self.player.x) * 0.0003;
        }
        if self.geometry_blend > 0.7 {
            if self.player.x > 2.5 {
                self.player.x = -2.5;
                self.runtime.particles.emit_burst(self.player.x, self.player.y, RED, 10);
                self.runtime.effects.chromatic(2.0, 0.3);
            }
            if self.player.x < -2.5 {
                self.player.x = 2.5;
                self.runtime.particles.emit_burst(self.player.x, self.player.y, RED, 10);
                self.runtime.effects.chromatic(2.0, 0.3);
            }
        }
        self.player.x += dx;
        self.player.y = (self.player.y + dy).clamp(-0.9, 0.9);

        // === PORTAL LOGIC ===
        if self.portal_cooldown <= 0.0 {
            let mut teleported = false;

            // 1. Auto-teleport portals (check every frame — just walk through!)
            for i in 0..self.portals.len() {
                if teleported { break; }
                if self.portals[i].portal_type.is_auto() {
                    if self.try_teleport(i, true) {
                        teleported = true;
                        break;
                    }
                    if self.try_teleport(i, false) {
                        teleported = true;
                        break;
                    }
                }
            }

            // 2. Action-key portals (Space/E)
            if !teleported && action {
                for i in 0..self.portals.len() {
                    if teleported { break; }
                    if self.try_teleport(i, true) {
                        teleported = true;
                        break;
                    }
                    if self.try_teleport(i, false) {
                        teleported = true;
                        break;
                    }
                }
            }

            if teleported {
                self.geometry_blend = self.get_blend_at(self.player.x);
            }
        }

        for portal in &mut self.portals { portal.update(dt); }

        // Ambient particles
        if rand::gen_range(0.0_f32, 1.0) < 0.12 {
            let px = self.player.x + rand::gen_range(-2.0, 2.0);
            let py = self.player.y + rand::gen_range(-1.0, 1.0);
            let c = self.blend_color();
            self.runtime.particles.emit(crate::engine::particles::Particle::new(px, py,
                Color::new(c.r * 3.0, c.g * 3.0, c.b * 3.0, 0.5)));
        }

        for c in &mut self.collectibles { c.update(dt); }
        for c in &mut self.collectibles {
            if !c.collected {
                if self.player.distance_to(c.pos) < 0.08 {
                    c.collected = true; self.collected_count += 1;
                    self.runtime.combo.on_collect(self.runtime.level_time, 200);
                    self.runtime.particles.emit_burst(c.pos.x, c.pos.y, c.color, 15);
                    self.runtime.effects.shake(3.0, 0.15);
                }
            }
        }

        if self.collected_count >= self.total_collectibles && !self.completed {
            self.completed = true; self.completion_timer = 0.0;
            self.runtime.effects.flash(WHITE, 0.5);
            self.runtime.effects.shake(6.0, 0.6);
        }
        LevelResult::Playing
    }

    fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        let cx = self.player.x;
        let cy = self.player.y;
        let t = self.runtime.level_time;

        clear_background(self.blend_color());

        // Zone labels
        renderer::draw_text_world("HYPERBOLIC", -2.0, -0.95, cx, cy, self.zoom, sw, sh, 0.08, Color::new(0.3, 0.5, 1.0, 0.4));
        renderer::draw_text_world("EUCLIDEAN", 0.0, -0.95, cx, cy, self.zoom, sw, sh, 0.08, Color::new(0.3, 1.0, 0.3, 0.4));
        renderer::draw_text_world("SPHERICAL", 2.0, -0.95, cx, cy, self.zoom, sw, sh, 0.08, Color::new(1.0, 0.3, 0.3, 0.4));

        let boundary_color = Color::new(0.4, 0.4, 0.4, 0.3);
        renderer::draw_line_world(-1.0, -1.5, -1.0, 1.5, cx, cy, self.zoom, sw, sh, boundary_color, 2.0);
        renderer::draw_line_world(1.0, -1.5, 1.0, 1.5, cx, cy, self.zoom, sw, sh, boundary_color, 2.0);

        // Grid in hyperbolic zone (geodesic arcs)
        let grid_color = Color::new(0.1, 0.1, 0.15, 0.3);
        for ring in 1..=5 {
            let r = ring as f32 * 0.15;
            for i in 0..24 {
                let a1 = std::f32::consts::TAU * i as f32 / 24.0;
                let a2 = std::f32::consts::TAU * (i + 1) as f32 / 24.0;
                let x1 = -2.0 + r * a1.cos(); let y1 = r * a1.sin();
                let x2 = -2.0 + r * a2.cos(); let y2 = r * a2.sin();
                if x1 > -3.0 && x1 < -0.5 {
                    illusion::draw_geodesic_line(x1, y1, x2, y2, cx, cy, self.zoom, sw, sh, grid_color, 1.0, 6);
                }
            }
        }
        // Euclidean zone grid
        for i in -5..=5 {
            let x = i as f32 * 0.15;
            if x >= -0.5 && x <= 0.5 {
                renderer::draw_line_world(x, -1.0, x, 1.0, cx, cy, self.zoom, sw, sh, grid_color, 1.0);
                renderer::draw_line_world(-0.5, x, 0.5, x, cx, cy, self.zoom, sw, sh, grid_color, 1.0);
            }
        }

        // Walls
        for wall in &self.walls {
            renderer::draw_line_world(wall.p1.x, wall.p1.y, wall.p2.x, wall.p2.y,
                cx, cy, self.zoom, sw, sh, wall.color, 3.0);
        }

        // Enemies
        for e in &self.runtime.enemies { renderer::draw_enemy(e, cx, cy, self.zoom, sw, sh); }

        // === CURVATURE FIELDS IN EACH ZONE ===
        // Hyperbolic zone: diverging field
        illusion::draw_curvature_field(-2.0, 0.3, 0.4, -1.0,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.2, 0.4, 1.0, 0.25), t);
        // Spherical zone: converging field
        illusion::draw_curvature_field(2.0, 0.3, 0.4, 1.0,
            cx, cy, self.zoom, sw, sh,
            Color::new(1.0, 0.3, 0.2, 0.25), t);

        // === PARALLEL LINE DEMOS IN EACH ZONE ===
        // Hyperbolic: diverge
        illusion::draw_parallel_deviation(-2.0, -0.75, std::f32::consts::FRAC_PI_2,
            0.04, 0.2, -0.7,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.3, 0.6, 1.0, 0.5), t);
        // Spherical: converge
        illusion::draw_parallel_deviation(2.0, -0.75, std::f32::consts::FRAC_PI_2,
            0.04, 0.2, 0.7,
            cx, cy, self.zoom, sw, sh,
            Color::new(1.0, 0.4, 0.2, 0.5), t);

        // === PORTAL RENDERING (using all engine features) ===
        let window_data = self.collectibles_for_window();

        for (_portal_idx, portal) in self.portals.iter().enumerate() {
            let pulse = (portal.angle).sin() * 0.01;

            // 1. GEODESIC LINK (curved arc connecting A to B)
            if portal.show_geodesic_link {
                renderer::draw_portal_geodesic_link(
                    portal.pos_a.x, portal.pos_a.y,
                    portal.pos_b.x, portal.pos_b.y,
                    cx, cy, self.zoom, sw, sh,
                    Color::new(
                        (portal.color_a.r + portal.color_b.r) / 2.0,
                        (portal.color_a.g + portal.color_b.g) / 2.0,
                        (portal.color_a.b + portal.color_b.b) / 2.0,
                        0.4),
                    1.0, t, 24);
            }

            // 2. WORMHOLE CORRIDOR (between auto-warp portals)
            if portal.show_corridor {
                illusion::draw_wormhole_corridor(
                    portal.pos_a.x, portal.pos_a.y,
                    portal.pos_b.x, portal.pos_b.y,
                    portal.radius * 1.5,
                    cx, cy, self.zoom, sw, sh,
                    Color::new(0.0, 0.7, 1.0, 0.4), t);
            }

            // 3. GRAVITY LENS (around every portal)
            renderer::draw_gravity_lens(
                portal.pos_a.x, portal.pos_a.y, portal.radius * 1.8,
                cx, cy, self.zoom, sw, sh,
                portal.color_a, t);
            renderer::draw_gravity_lens(
                portal.pos_b.x, portal.pos_b.y, portal.radius * 1.8,
                cx, cy, self.zoom, sw, sh,
                portal.color_b, t);

            // Render each side (A and B)
            for &(side_x, side_y, side_color, side_idx) in &[
                (portal.pos_a.x, portal.pos_a.y, portal.color_a, 0_usize),
                (portal.pos_b.x, portal.pos_b.y, portal.color_b, 1_usize),
            ] {
                // 4. PORTAL WARP (spatial compression rings + spiral)
                if portal.show_warp {
                    illusion::draw_portal_warp(
                        side_x, side_y, portal.radius,
                        &self.portal_illusion, side_color,
                        cx, cy, self.zoom, sw, sh);
                }

                // 5. RECURSIVE PORTAL (infinite regression)
                if portal.portal_type == PortalType::Recursive {
                    illusion::draw_recursive_portal(
                        side_x, side_y, portal.radius,
                        // The "target" is the other side
                        if side_idx == 0 { portal.pos_b.x } else { portal.pos_a.x },
                        if side_idx == 0 { portal.pos_b.y } else { portal.pos_a.y },
                        side_color, t, 4,
                        cx, cy, self.zoom, sw, sh);
                }

                // 6. PORTAL WINDOW (see-through illusion)
                if portal.show_window {
                    let target_x = if side_idx == 0 { portal.pos_b.x } else { portal.pos_a.x };
                    let target_y = if side_idx == 0 { portal.pos_b.y } else { portal.pos_a.y };
                    illusion::draw_portal_window(
                        side_x, side_y, portal.radius * 0.8,
                        target_x, target_y,
                        &window_data, cx, cy, self.zoom, sw, sh, t);
                }

                // Outline
                renderer::draw_circle_outline_world(side_x, side_y, portal.radius + pulse,
                    cx, cy, self.zoom, sw, sh, side_color, 2.0);

                // Label
                let label_text = if portal.label.is_empty() { "P" } else { &portal.label };
                renderer::draw_text_world(label_text, side_x - 0.015, side_y - 0.015,
                    cx, cy, self.zoom, sw, sh, 0.03, WHITE);
            }

            // Type-specific decorations
            match portal.portal_type {
                PortalType::OneWayAB => {
                    renderer::draw_text_world("->", portal.pos_a.x + portal.radius, portal.pos_a.y - 0.01,
                        cx, cy, self.zoom, sw, sh, 0.03, Color::new(1.0, 1.0, 0.5, 0.6));
                    renderer::draw_text_world("X", portal.pos_b.x + portal.radius, portal.pos_b.y - 0.01,
                        cx, cy, self.zoom, sw, sh, 0.03, Color::new(1.0, 0.3, 0.3, 0.5));
                }
                PortalType::OneWayBA => {
                    renderer::draw_text_world("X", portal.pos_a.x + portal.radius, portal.pos_a.y - 0.01,
                        cx, cy, self.zoom, sw, sh, 0.03, Color::new(1.0, 0.3, 0.3, 0.5));
                    renderer::draw_text_world("<-", portal.pos_b.x + portal.radius, portal.pos_b.y - 0.01,
                        cx, cy, self.zoom, sw, sh, 0.03, Color::new(1.0, 1.0, 0.5, 0.6));
                }
                PortalType::Mirror => {
                    renderer::draw_text_world("<M>", portal.pos_a.x - 0.03, portal.pos_a.y + portal.radius,
                        cx, cy, self.zoom, sw, sh, 0.02, Color::new(0.5, 0.8, 1.0, 0.5));
                    renderer::draw_text_world("M>", portal.pos_b.x - 0.02, portal.pos_b.y + portal.radius,
                        cx, cy, self.zoom, sw, sh, 0.02, Color::new(1.0, 0.8, 0.5, 0.5));
                }
                PortalType::GeometryShift => {
                    renderer::draw_text_world("~G~", portal.pos_a.x - 0.03, portal.pos_a.y + portal.radius,
                        cx, cy, self.zoom, sw, sh, 0.02, Color::new(0.0, 1.0, 0.5, 0.5));
                    renderer::draw_text_world("~G~", portal.pos_b.x - 0.03, portal.pos_b.y + portal.radius,
                        cx, cy, self.zoom, sw, sh, 0.02, Color::new(1.0, 0.5, 0.0, 0.5));
                }
                PortalType::AutoWarp => {
                    // Pulsing "WALK IN" label
                    let alpha = 0.4 + (t * 3.0).sin() * 0.3;
                    renderer::draw_text_world("WALK IN", portal.pos_a.x - 0.02, portal.pos_a.y + portal.radius,
                        cx, cy, self.zoom, sw, sh, 0.018, Color::new(0.0, 0.7, 1.0, alpha));
                    renderer::draw_text_world("WALK IN", portal.pos_b.x - 0.02, portal.pos_b.y + portal.radius,
                        cx, cy, self.zoom, sw, sh, 0.018, Color::new(1.0, 0.5, 0.0, alpha));
                }
                PortalType::Recursive => {
                    let alpha = 0.4 + (t * 2.0).sin() * 0.2;
                    renderer::draw_text_world("INFINITE", portal.pos_a.x - 0.025, portal.pos_a.y + portal.radius,
                        cx, cy, self.zoom, sw, sh, 0.018, Color::new(0.8, 0.0, 1.0, alpha));
                    renderer::draw_text_world("INFINITE", portal.pos_b.x - 0.025, portal.pos_b.y + portal.radius,
                        cx, cy, self.zoom, sw, sh, 0.018, Color::new(0.0, 1.0, 0.8, alpha));
                }
                PortalType::Normal => {}
                PortalType::DimensionalShift => {}
            }
        }

        // Particles
        self.runtime.particles.draw(cx, cy, self.zoom, sw, sh);

        // === IMPOSSIBLE GEOMETRY DECORATIONS ===
        for &(dx, dy, ds) in &self.deco_tesseracts {
            let tc = self.blend_color();
            illusion::draw_tesseract(dx, dy, ds, cx, cy, self.zoom, sw, sh,
                Color::new(tc.r * 3.0 + 0.3, tc.g * 3.0 + 0.2, tc.b * 3.0 + 0.4, 0.5), t);
        }
        for &(dx, dy, ds) in &self.deco_triangles {
            illusion::draw_impossible_triangle(dx, dy, ds, cx, cy, self.zoom, sw, sh,
                Color::new(0.6, 0.3, 0.8, 0.35), t);
        }
        for &(dx, dy, ds) in &self.deco_staircases {
            illusion::draw_impossible_staircase(dx, dy, ds, cx, cy, self.zoom, sw, sh,
                Color::new(0.4, 0.6, 0.9, 0.3), t);
        }

        // NEW v5.0: Escher grid and Klein bottle decorations
        illusion::draw_escher_grid(0.0, -0.7, 0.15, 4,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.5, 0.3, 0.8, 0.25), t);

        illusion::draw_klein_bottle(1.5, 0.0, 0.18,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.8, 0.3, 0.5, 0.5), t);

        illusion::draw_mirror_room(-1.5, 0.5, 0.18,
            cx, cy, self.zoom, sw, sh,
            Color::new(0.3, 0.5, 0.8, 0.4), t, self.geometry_blend * 2.0 - 1.0);

        // Collectibles
        for c in &self.collectibles {
            if !c.collected {
                let er = c.effective_radius();
                let by = c.bob_y();
                renderer::draw_circle_world(c.pos.x, c.pos.y + by, er * 2.0, cx, cy, self.zoom, sw, sh,
                    Color::new(c.color.r, c.color.g, c.color.b, 0.2));
                renderer::draw_circle_world(c.pos.x, c.pos.y + by, er, cx, cy, self.zoom, sw, sh, c.color);
                renderer::draw_text_world(&c.label.to_string(), c.pos.x - 0.01, c.pos.y + by - 0.01,
                    cx, cy, self.zoom, sw, sh, 0.03, WHITE);
            }
        }

        // Trail
        let pc = self.blend_color();
        let trail_color = Color::new((pc.r * 5.0).min(1.0), (pc.g * 5.0).min(1.0) + 0.3, (pc.b * 5.0).min(1.0), 1.0);
        renderer::draw_trail(&self.runtime.dash.trail, cx, cy, self.zoom, sw, sh, trail_color);

        // Player
        let player_r = 0.025;
        let invuln_flash = if self.runtime.invuln_timer > 0.0 {
            (self.runtime.invuln_timer * 10.0).sin().abs() > 0.3
        } else { false };
        if !invuln_flash {
            let player_color = Color::new((pc.r * 5.0).min(1.0), (pc.g * 5.0).min(1.0) + 0.3, (pc.b * 5.0).min(1.0), 1.0);
            renderer::draw_circle_world(self.player.x, self.player.y, player_r * 2.0, cx, cy, self.zoom, sw, sh,
                Color::new(player_color.r, player_color.g, player_color.b, 0.2));
            renderer::draw_circle_world(self.player.x, self.player.y, player_r, cx, cy, self.zoom, sw, sh, player_color);
        }

        // Proximity indicators
        let half_w = sw * 0.45 * self.zoom;
        let half_h = sw * 0.45 * self.zoom;
        for c in &self.collectibles {
            if !c.collected {
                let sx = sw / 2.0 + (c.pos.x - cx) * half_w;
                let sy = sh / 2.0 + (c.pos.y - cy) * half_h;
                renderer::draw_proximity_indicator(sw / 2.0, sh / 2.0, sx, sy, sw, sh, c.color, &c.label.to_string());
            }
        }

        // Geometry blend bar
        let bar_y = sh - 30.0;
        let bar_w = sw * 0.4;
        let bar_x = (sw - bar_w) / 2.0;
        draw_rectangle(bar_x, bar_y, bar_w, 15.0, Color::new(0.1, 0.1, 0.1, 0.7));
        let fill_w = bar_w * self.geometry_blend;
        draw_rectangle(bar_x, bar_y, fill_w, 15.0, Color::new(0.3, 0.5, 0.8, 0.5));
        draw_text("HYP", bar_x - 35.0, bar_y, 14.0, Color::new(0.0, 0.9, 0.9, 1.0));
        draw_text("SPH", bar_x + bar_w + 5.0, bar_y, 14.0, RED);

        // Portal type legend
        renderer::draw_text_world("Portal Types: N=Normal M=Mirror A>B=OneWay G=GeomShift W=AutoWarp R=Recursive",
            self.player.x - 1.8, self.player.y + 0.78,
            cx, cy, self.zoom, sw, sh, 0.018, Color::new(0.5, 0.5, 0.6, 0.4));

        if self.completed {
            let flash = (self.completion_timer * 5.0).sin().abs();
            renderer::draw_circle_world(self.player.x, self.player.y, 0.12 + flash * 0.05,
                cx, cy, self.zoom, sw, sh, Color::new(1.0, 1.0, 1.0, 0.3));
        }

        self.runtime.effects.draw_flash();
    }
}