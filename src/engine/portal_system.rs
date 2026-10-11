use macroquad::prelude::*;
use crate::geometry::GeoPoint;
use crate::engine::game::{Portal, PortalType, LevelRuntime};
use crate::engine::renderer;
use crate::engine::illusion;

/// Centralized portal management system.
/// Handles all portal logic: updating, checking teleportation, and rendering.
pub struct PortalSystem {
    pub portals: Vec<Portal>,
    pub global_cooldown: f32,
}

impl PortalSystem {
    pub fn new() -> Self {
        Self { portals: Vec::new(), global_cooldown: 0.0 }
    }

    pub fn with_portals(portals: Vec<Portal>) -> Self {
        Self { portals, global_cooldown: 0.0 }
    }

    pub fn add(&mut self, portal: Portal) {
        self.portals.push(portal);
    }

    /// Update all portal animations.
    pub fn update(&mut self, dt: f32) {
        for portal in &mut self.portals {
            portal.update(dt);
        }
        if self.global_cooldown > 0.0 {
            self.global_cooldown -= dt;
        }
    }

    /// Check and handle teleportation. Returns true if the player was teleported.
    /// `player` is mutable ref — will be moved if teleported.
    /// `from_a` indicates whether to try entering from the A side first.
    pub fn try_teleport(&mut self, player: &mut GeoPoint, action: bool, runtime: &mut LevelRuntime) -> bool {
        if self.global_cooldown > 0.0 { return false; }

        let mut teleported = false;

        // 1. Auto-teleport portals (walk through)
        for i in 0..self.portals.len() {
            if teleported { break; }
            if self.portals[i].portal_type.is_auto() {
                if Self::try_single_portal(&self.portals[i], player, true, runtime) {
                    teleported = true;
                    // Update the portal's cooldown
                    self.portals[i].cooldown = 1.0;
                }
            }
        }

        // 2. Action-key portals
        if !teleported && action {
            for i in 0..self.portals.len() {
                if teleported { break; }
                if Self::try_single_portal(&self.portals[i], player, true, runtime) {
                    teleported = true;
                    self.portals[i].cooldown = 1.0;
                }
                if !teleported && Self::try_single_portal(&self.portals[i], player, false, runtime) {
                    teleported = true;
                    self.portals[i].cooldown = 1.0;
                }
            }
        }

        if teleported {
            self.global_cooldown = 0.5;
        }
        teleported
    }

    fn try_single_portal(portal: &Portal, player: &mut GeoPoint, from_a: bool, runtime: &mut LevelRuntime) -> bool {
        let (enter_pos, exit_pos) = if from_a {
            (portal.pos_a, portal.pos_b)
        } else {
            (portal.pos_b, portal.pos_a)
        };

        let dist = player.distance_to(enter_pos);
        if dist >= portal.radius + 0.08 { return false; }

        // Check direction permission
        if from_a && !portal.can_enter_from_a() { return false; }
        if !from_a && !portal.can_enter_from_b() { return false; }

        if portal.cooldown > 0.0 { return false; }

        // Mirror flip (compute before teleport)
        let mut dest = exit_pos;
        if portal.mirror_flip_x {
            dest.x = 2.0 * exit_pos.x - player.x;
        }

        *player = dest;

        // Effects
        let entry_color = if from_a { portal.color_a } else { portal.color_b };
        let exit_color = if from_a { portal.color_b } else { portal.color_a };
        let entry_pos_f = if from_a { portal.pos_a } else { portal.pos_b };
        let exit_pos_f = if from_a { portal.pos_b } else { portal.pos_a };

        runtime.particles.emit_burst(entry_pos_f.x, entry_pos_f.y, entry_color, 20);
        runtime.particles.emit_burst(exit_pos_f.x, exit_pos_f.y, exit_color, 20);
        runtime.effects.flash(Color::new(0.5, 0.0, 1.0, 1.0), 0.2);
        runtime.effects.chromatic(4.0, 0.5);
        runtime.effects.shake(4.0, 0.2);

        match portal.portal_type {
            PortalType::GeometryShift => {
                runtime.effects.flash(Color::new(0.0, 1.0, 0.5, 1.0), 0.3);
                runtime.effects.shake(6.0, 0.4);
            }
            PortalType::Recursive => {
                runtime.effects.flash(Color::new(0.8, 0.0, 1.0, 1.0), 0.25);
                runtime.effects.shake(5.0, 0.3);
            }
            PortalType::DimensionalShift => {
                runtime.effects.flash(Color::new(1.0, 0.5, 0.0, 1.0), 0.4);
                runtime.effects.shake(8.0, 0.5);
                runtime.effects.chromatic(8.0, 0.8);
            }
            _ => {}
        }

        true
    }

    /// Draw all portals with full engine effects.
    pub fn draw(&self, cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32,
                t: f32, portal_illusion: &illusion::PortalIllusion) {
        for portal in &self.portals {
            let pulse = (portal.angle).sin() * 0.01;

            // 1. Geodesic link
            if portal.show_geodesic_link {
                renderer::draw_portal_geodesic_link(
                    portal.pos_a.x, portal.pos_a.y,
                    portal.pos_b.x, portal.pos_b.y,
                    cam_x, cam_y, zoom, sw, sh,
                    Color::new(
                        (portal.color_a.r + portal.color_b.r) / 2.0,
                        (portal.color_a.g + portal.color_b.g) / 2.0,
                        (portal.color_a.b + portal.color_b.b) / 2.0,
                        0.4),
                    1.0, t, 24);
            }

            // 2. Wormhole corridor
            if portal.show_corridor {
                illusion::draw_wormhole_corridor(
                    portal.pos_a.x, portal.pos_a.y,
                    portal.pos_b.x, portal.pos_b.y,
                    portal.radius * 1.5,
                    cam_x, cam_y, zoom, sw, sh,
                    Color::new(0.0, 0.7, 1.0, 0.4), t);
            }

            // 3. Each side
            for &(sx, sy, sc, side_idx) in &[
                (portal.pos_a.x, portal.pos_a.y, portal.color_a, 0_usize),
                (portal.pos_b.x, portal.pos_b.y, portal.color_b, 1_usize),
            ] {
                // Gravity lens
                renderer::draw_gravity_lens(sx, sy, portal.radius * 1.8,
                    cam_x, cam_y, zoom, sw, sh, sc, t);

                // Portal warp
                if portal.show_warp {
                    illusion::draw_portal_warp(sx, sy, portal.radius,
                        portal_illusion, sc, cam_x, cam_y, zoom, sw, sh);
                }

                // Recursive
                if portal.portal_type == PortalType::Recursive {
                    let tx = if side_idx == 0 { portal.pos_b.x } else { portal.pos_a.x };
                    let ty = if side_idx == 0 { portal.pos_b.y } else { portal.pos_a.y };
                    illusion::draw_recursive_portal(sx, sy, portal.radius, tx, ty,
                        sc, t, 4, cam_x, cam_y, zoom, sw, sh);
                }

                // Portal body
                renderer::draw_circle_world(sx, sy, portal.radius + pulse,
                    cam_x, cam_y, zoom, sw, sh,
                    Color::new(sc.r * 0.3, sc.g * 0.3, sc.b * 0.3, 0.4));
                renderer::draw_circle_outline_world(sx, sy, portal.radius + pulse,
                    cam_x, cam_y, zoom, sw, sh, sc, 2.0);

                // Label
                renderer::draw_text_world(&portal.label, sx - 0.02, sy - portal.radius - 0.03,
                    cam_x, cam_y, zoom, sw, sh, 0.025, sc);

                // Portal preview window (shows direction to exit)
                if side_idx == 0 {
                    renderer::draw_portal_preview(
                        portal.pos_a.x, portal.pos_a.y,
                        portal.pos_b.x, portal.pos_b.y,
                        portal.radius, t,
                        cam_x, cam_y, zoom, sw, sh,
                        portal.color_b,
                    );
                }

                // Cooldown indicator
                if portal.cooldown > 0.0 {
                    renderer::draw_portal_cooldown(
                        sx, sy, portal.radius, portal.cooldown, 1.0,
                        cam_x, cam_y, zoom, sw, sh,
                    );
                }

                // Direction arrows for one-way portals
                match portal.portal_type {
                    PortalType::OneWayAB => {
                        if side_idx == 0 {
                            Self::draw_arrow(sx, sy, portal.radius, sc, cam_x, cam_y, zoom, sw, sh, t);
                        }
                    }
                    PortalType::OneWayBA => {
                        if side_idx == 1 {
                            Self::draw_arrow(sx, sy, portal.radius, sc, cam_x, cam_y, zoom, sw, sh, t);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw_arrow(sx: f32, sy: f32, radius: f32, color: Color,
                  cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32, t: f32) {
        let offset = radius + 0.02 + (t * 2.0).sin() * 0.005;
        renderer::draw_text_world(">", sx + offset, sy - 0.01,
            cam_x, cam_y, zoom, sw, sh, 0.03,
            Color::new(color.r, color.g, color.b, 0.6 + (t * 3.0).sin() * 0.2));
    }
}