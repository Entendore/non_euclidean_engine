use macroquad::prelude::*;

/// A floating score/damage/info text that rises and fades.
pub struct FloatingText {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub text: String,
    pub color: Color,
    pub font_size: f32,
    pub life: f32,
    pub max_life: f32,
    pub scale: f32,
}

impl FloatingText {
    pub fn new(x: f32, y: f32, text: &str, color: Color, font_size: f32, duration: f32) -> Self {
        Self {
            x, y,
            vy: -0.03, // Rise upward
            text: text.to_string(),
            color,
            font_size,
            life: duration,
            max_life: duration,
            scale: 1.5, // Pop in effect
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.life -= dt;
        self.y += self.vy * dt;
        self.vy *= 0.98; // Slow down
        // Scale pops in then settles
        if self.scale > 1.0 {
            self.scale -= dt * 3.0;
            if self.scale < 1.0 { self.scale = 1.0; }
        }
    }

    pub fn alive(&self) -> bool {
        self.life > 0.0
    }

    pub fn alpha(&self) -> f32 {
        if self.life > self.max_life * 0.7 {
            1.0
        } else {
            (self.life / (self.max_life * 0.7)).clamp(0.0, 1.0)
        }
    }
}

/// Manages floating text popups in world space.
pub struct FloatingTextSystem {
    texts: Vec<FloatingText>,
}

impl FloatingTextSystem {
    pub fn new() -> Self {
        Self { texts: Vec::with_capacity(30) }
    }

    /// Add a floating text at a world position.
    pub fn add(&mut self, x: f32, y: f32, text: &str, color: Color) {
        self.add_with_duration(x, y, text, color, 1.5);
    }

    pub fn add_with_duration(&mut self, x: f32, y: f32, text: &str, color: Color, duration: f32) {
        self.texts.push(FloatingText::new(x, y, text, color, 20.0, duration));
    }

    /// Convenience: score popup
    pub fn add_score(&mut self, x: f32, y: f32, score: u32, combo: u32) {
        let text = if combo > 1 {
            format!("+{} (x{})", score, combo)
        } else {
            format!("+{}", score)
        };
        let color = if combo >= 4 { Color::new(1.0, 0.8, 0.0, 1.0) }
            else if combo >= 2 { Color::new(0.5, 1.0, 0.5, 1.0) }
            else { Color::new(1.0, 1.0, 1.0, 1.0) };
        self.texts.push(FloatingText::new(x, y + 0.03, &text, color, 22.0, 1.8));
    }

    /// Convenience: damage popup
    pub fn add_damage(&mut self, x: f32, y: f32) {
        self.texts.push(FloatingText::new(x, y + 0.03, "HIT!", Color::new(1.0, 0.2, 0.2, 1.0), 24.0, 1.0));
    }

    /// Convenience: info text (white, smaller)
    pub fn add_info(&mut self, x: f32, y: f32, text: &str) {
        self.texts.push(FloatingText::new(x, y, text, Color::new(0.8, 0.8, 0.9, 1.0), 14.0, 2.5));
    }

    pub fn update(&mut self, dt: f32) {
        for t in &mut self.texts { t.update(dt); }
        self.texts.retain(|t| t.alive());
    }

    /// Draw all floating texts in world space.
    pub fn draw(&self, cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32) {
        use crate::engine::renderer::draw_text_world;
        for t in &self.texts {
            let alpha = t.alpha();
            let size = t.font_size * t.scale;
            draw_text_world(
                &t.text, t.x, t.y,
                cam_x, cam_y, zoom, sw, sh,
                size,
                Color::new(t.color.r, t.color.g, t.color.b, alpha),
            );
        }
    }

    pub fn clear(&mut self) {
        self.texts.clear();
    }
}

// ===================== FPS COUNTER =====================

/// Simple FPS counter with smoothed display.
pub struct FpsCounter {
    frame_times: Vec<f64>,
    display_fps: f32,
    update_timer: f32,
    pub visible: bool,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            frame_times: Vec::with_capacity(60),
            display_fps: 60.0,
            update_timer: 0.0,
            visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn update(&mut self, dt: f64) {
        self.frame_times.push(dt);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        self.update_timer += dt as f32;
        if self.update_timer >= 0.5 {
            self.update_timer = 0.0;
            if !self.frame_times.is_empty() {
                let avg = self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
                self.display_fps = (1.0 / avg) as f32;
            }
        }
    }

    pub fn draw(&self) {
        if !self.visible { return; }
        let fps_text = format!("FPS: {:.0}", self.display_fps);
        let color = if self.display_fps >= 50.0 { Color::new(0.0, 1.0, 0.0, 0.7) }
            else if self.display_fps >= 30.0 { Color::new(1.0, 1.0, 0.0, 0.7) }
            else { Color::new(1.0, 0.0, 0.0, 0.7) };
        draw_text(&fps_text, 10.0, screen_height() - 25.0, 14.0, color);
    }
}

// ===================== ENHANCED MINIMAP =====================

/// Enhanced minimap with geometry-specific overlays.
pub struct EnhancedMinimap {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub show_geometry_overlay: bool,
    pub border_color: Color,
}

impl EnhancedMinimap {
    pub fn new() -> Self {
        Self {
            x: 10.0,
            y: 10.0,
            size: 120.0,
            show_geometry_overlay: true,
            border_color: Color::new(0.3, 0.4, 0.6, 0.8),
        }
    }

    /// Position the minimap at bottom-left with offset from HUD.
    pub fn position_hud(&mut self, sw: f32, sh: f32) {
        self.x = 10.0;
        self.y = sh - self.size - 40.0;
    }

    /// Draw minimap background and border.
    pub fn draw_background(&self, geo_type: &crate::geometry::GeometryType) {
        // Semi-transparent background
        draw_rectangle(self.x, self.y, self.size, self.size, Color::new(0.0, 0.0, 0.0, 0.6));
        
        // Geometry-specific overlay
        if self.show_geometry_overlay {
            let cx = self.x + self.size / 2.0;
            let cy = self.y + self.size / 2.0;
            let r = self.size / 2.0 - 2.0;

            match geo_type {
                crate::geometry::GeometryType::Hyperbolic |
                crate::geometry::GeometryType::Hyperbolic4D |
                crate::geometry::GeometryType::Hyperbolic5D => {
                    // Poincaré disk boundary circle
                    draw_circle_lines(cx, cy, r, 1.0, Color::new(0.0, 0.4, 0.6, 0.4));
                    // Concentric conformal guide circles
                    for i in 1..4 {
                        let cr = r * i as f32 / 4.0;
                        draw_circle_lines(cx, cy, cr, 1.0, Color::new(0.0, 0.2, 0.3, 0.15));
                    }
                }
                crate::geometry::GeometryType::Spherical => {
                    // Spherical grid lines
                    draw_rectangle_lines(self.x, self.y, self.size, self.size, 1.0, 
                        Color::new(0.6, 0.3, 0.0, 0.4));
                    // Latitude lines
                    for i in 1..4 {
                        let ly = self.y + self.size * i as f32 / 4.0;
                        draw_line(self.x, ly, self.x + self.size, ly, 1.0,
                            Color::new(0.4, 0.2, 0.0, 0.2));
                    }
                }
                crate::geometry::GeometryType::Hyperspherical4D |
                crate::geometry::GeometryType::Hyperspherical5D => {
                    // Hyperspace grid with dimension indicators
                    draw_rectangle_lines(self.x, self.y, self.size, self.size, 1.0,
                        Color::new(0.4, 0.1, 0.7, 0.4));
                    // Diagonal cross for extra dimensions
                    draw_line(self.x, self.y, self.x + self.size, self.y + self.size, 1.0,
                        Color::new(0.3, 0.1, 0.5, 0.2));
                    draw_line(self.x + self.size, self.y, self.x, self.y + self.size, 1.0,
                        Color::new(0.3, 0.1, 0.5, 0.2));
                }
                _ => {
                    draw_rectangle_lines(self.x, self.y, self.size, self.size, 1.0,
                        Color::new(0.3, 0.3, 0.3, 0.4));
                }
            }
        }

        // Border
        draw_rectangle_lines(self.x, self.y, self.size, self.size, 1.5, self.border_color);
    }

    /// Draw player position on the minimap.
    pub fn draw_player(&self, player_x: f32, player_y: f32, 
                       world_range: f32, geo_type: &crate::geometry::GeometryType) {
        let cx = self.x + self.size / 2.0;
        let cy = self.y + self.size / 2.0;
        let r = self.size / 2.0 - 4.0;

        let (mx, my) = match geo_type {
            crate::geometry::GeometryType::Hyperbolic |
            crate::geometry::GeometryType::Hyperbolic4D |
            crate::geometry::GeometryType::Hyperbolic5D => {
                // Poincaré disk: scale to circle
                let scale = r * 0.95;
                (cx + player_x * scale, cy + player_y * scale)
            }
            crate::geometry::GeometryType::Spherical => {
                // Spherical: wrap to square
                let sx = ((player_x / world_range + 1.0) * 0.5).clamp(0.0, 1.0);
                let sy = ((player_y / world_range + 1.0) * 0.5).clamp(0.0, 1.0);
                (self.x + 3.0 + sx * (self.size - 6.0), self.y + 3.0 + sy * (self.size - 6.0))
            }
            _ => {
                // Euclidean/hyperspace: linear mapping
                let sx = (player_x / world_range + 1.0) * 0.5;
                let sy = (player_y / world_range + 1.0) * 0.5;
                (cx + (sx - 0.5) * r * 2.0, cy + (sy - 0.5) * r * 2.0)
            }
        };

        draw_circle(mx, my, 3.0, WHITE);
        // Pulsing ring
        let pulse = (get_time() * 4.0).sin() * 0.5 + 0.5;
        draw_circle_lines(mx, my, 5.0 + pulse * 2.0, 1.0, 
            Color::new(1.0, 1.0, 1.0, 0.3 + pulse * 0.3));
    }

    /// Draw collectible dots on the minimap.
    pub fn draw_collectibles(&self, items: &[(f32, f32, bool, char)], 
                             world_range: f32,
                             geo_type: &crate::geometry::GeometryType) {
        let cx = self.x + self.size / 2.0;
        let cy = self.y + self.size / 2.0;
        let r = self.size / 2.0 - 4.0;

        for &(ix, iy, collected, _) in items {
            if collected { continue; }
            let (mx, my) = match geo_type {
                crate::geometry::GeometryType::Hyperbolic |
                crate::geometry::GeometryType::Hyperbolic4D |
                crate::geometry::GeometryType::Hyperbolic5D => {
                    let scale = r * 0.95;
                    (cx + ix * scale, cy + iy * scale)
                }
                _ => {
                    let sx = (ix / world_range + 1.0) * 0.5;
                    let sy = (iy / world_range + 1.0) * 0.5;
                    (cx + (sx - 0.5) * r * 2.0, cy + (sy - 0.5) * r * 2.0)
                }
            };
            draw_circle(mx, my, 2.0, Color::new(1.0, 0.9, 0.3, 0.8));
        }
    }

    /// Draw enemy positions on the minimap.
    pub fn draw_enemies(&self, enemies: &[(f32, f32)], world_range: f32,
                        geo_type: &crate::geometry::GeometryType) {
        let cx = self.x + self.size / 2.0;
        let cy = self.y + self.size / 2.0;
        let r = self.size / 2.0 - 4.0;

        for &(ex, ey) in enemies {
            let (mx, my) = match geo_type {
                crate::geometry::GeometryType::Hyperbolic |
                crate::geometry::GeometryType::Hyperbolic4D |
                crate::geometry::GeometryType::Hyperbolic5D => {
                    let scale = r * 0.95;
                    (cx + ex * scale, cy + ey * scale)
                }
                _ => {
                    let sx = (ex / world_range + 1.0) * 0.5;
                    let sy = (ey / world_range + 1.0) * 0.5;
                    (cx + (sx - 0.5) * r * 2.0, cy + (sy - 0.5) * r * 2.0)
                }
            };
            draw_circle(mx, my, 2.5, Color::new(1.0, 0.2, 0.2, 0.7));
        }
    }
}