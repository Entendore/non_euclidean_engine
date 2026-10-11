use macroquad::prelude::*;

/// Camera that handles viewport transformation for non-Euclidean spaces.
pub struct Camera {
    pub pos_x: f32,
    pub pos_y: f32,
    pub zoom: f32,
    pub rotation: f32,
    /// For hyperbolic: distortion center, for spherical: view center
    pub distortion: f32,
    screen_w: f32,
    screen_h: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            zoom: 1.0,
            rotation: 0.0,
            distortion: 1.0,
            screen_w: 800.0,
            screen_h: 600.0,
        }
    }

    pub fn update_screen_size(&mut self) {
        self.screen_w = screen_width();
        self.screen_h = screen_height();
    }

    /// Convert world coordinates to screen coordinates based on camera state.
    pub fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        let cx = self.screen_w / 2.0;
        let cy = self.screen_h / 2.0;
        let sx = cx + (wx - self.pos_x) * self.zoom * self.screen_w * 0.45;
        let sy = cy + (wy - self.pos_y) * self.zoom * self.screen_h * 0.45;
        (sx, sy)
    }

    /// Convert screen coordinates back to world coordinates.
    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        let cx = self.screen_w / 2.0;
        let cy = self.screen_h / 2.0;
        let wx = self.pos_x + (sx - cx) / (self.zoom * self.screen_w * 0.45);
        let wy = self.pos_y + (sy - cy) / (self.zoom * self.screen_h * 0.45);
        (wx, wy)
    }

    /// Get the visible world-space radius from the camera.
    pub fn view_radius(&self) -> f32 {
        1.0 / self.zoom
    }
}