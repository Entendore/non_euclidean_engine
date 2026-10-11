use macroquad::prelude::*;
use crate::geometry::GeometryType;

/// Manages smooth transitions between geometry types for blended levels.
pub struct GeometryTransition {
    /// Control points: (x_position, blend_value).
    /// blend_value: 0.0 = hyperbolic, 0.5 = euclidean, 1.0 = spherical
    pub zones: Vec<(f32, f32)>,
    pub current_blend: f32,
}

impl GeometryTransition {
    pub fn new(zones: Vec<(f32, f32)>) -> Self {
        Self { zones, current_blend: 0.5 }
    }

    /// Get the geometry blend at a given x position.
    pub fn blend_at(&self, x: f32) -> f32 {
        if self.zones.len() < 2 { return 0.5; }
        for i in 0..self.zones.len() - 1 {
            let (x0, b0) = self.zones[i];
            let (x1, b1) = self.zones[i + 1];
            if x >= x0 && x <= x1 {
                let t = (x - x0) / (x1 - x0);
                return b0 + (b1 - b0) * t;
            }
        }
        if x < self.zones[0].0 { return self.zones[0].1; }
        self.zones.last().map(|(_, b)| *b).unwrap_or(0.5)
    }

    /// Update the current blend based on player position.
    pub fn update(&mut self, player_x: f32) {
        self.current_blend = self.blend_at(player_x);
    }

    /// Get movement speed modifier based on current geometry.
    pub fn speed_modifier(&self) -> f32 {
        let hyperbolic_factor = 1.0 - self.current_blend;
        let spherical_factor = self.current_blend;
        1.0 + hyperbolic_factor * 0.5 - spherical_factor * 0.2
    }

    /// Get the effective geometry type at the current blend.
    pub fn effective_geometry(&self) -> GeometryType {
        if self.current_blend < 0.3 { GeometryType::Hyperbolic }
        else if self.current_blend > 0.7 { GeometryType::Spherical }
        else { GeometryType::Euclidean }
    }

    /// Background color based on blend.
    pub fn background_color(&self) -> Color {
        let b = self.current_blend;
        let r = if b > 0.5 { (b - 0.5) * 2.0 } else { 0.0 };
        let g = if b <= 0.5 { 1.0 - b * 2.0 } else { 1.0 - (b - 0.5) * 2.0 };
        let bl = if b < 0.5 { 1.0 - b * 2.0 } else { 0.0 };
        Color::new(r * 0.15, g * 0.08, bl * 0.15 + 0.03, 1.0)
    }

    /// Get the hyperbolic factor (0 = spherical, 1 = hyperbolic).
    pub fn hyperbolic_factor(&self) -> f32 {
        1.0 - self.current_blend
    }

    /// Get the spherical factor (0 = hyperbolic, 1 = spherical).
    pub fn spherical_factor(&self) -> f32 {
        self.current_blend
    }
}