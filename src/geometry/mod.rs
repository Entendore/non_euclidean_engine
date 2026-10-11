pub mod hyperbolic;
pub mod spherical;
pub mod euclidean;
pub mod hyperspace;

use macroquad::math::Vec2;

/// Represents a point in a geometric space (abstracted).
#[derive(Debug, Clone, Copy)]
pub struct GeoPoint {
    pub x: f32,
    pub y: f32,
}

impl GeoPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn from_vec2(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }

    pub fn distance_to(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

/// Represents a geodesic (shortest path) in a geometric space.
#[derive(Debug, Clone)]
pub struct Geodesic {
    pub start: GeoPoint,
    pub end: GeoPoint,
    pub curvature: f32,
}

/// All geometry types supported by the engine, including 4D and 5D spaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeometryType {
    Euclidean,
    Hyperbolic,
    Spherical,
    Hyperspherical4D,
    Hyperspherical5D,
    Hyperbolic4D,
    Hyperbolic5D,
}

impl GeometryType {
    /// Get the dimensionality of this geometry's underlying space.
    pub fn dimension(&self) -> usize {
        match self {
            GeometryType::Euclidean => 2,
            GeometryType::Hyperbolic => 2,
            GeometryType::Spherical => 2,
            GeometryType::Hyperspherical4D => 4,
            GeometryType::Hyperspherical5D => 5,
            GeometryType::Hyperbolic4D => 4,
            GeometryType::Hyperbolic5D => 5,
        }
    }

    /// Whether this geometry has constant positive curvature (spherical-like).
    pub fn is_positive_curvature(&self) -> bool {
        matches!(self, GeometryType::Spherical | GeometryType::Hyperspherical4D | GeometryType::Hyperspherical5D)
    }

    /// Whether this geometry has constant negative curvature (hyperbolic-like).
    pub fn is_negative_curvature(&self) -> bool {
        matches!(self, GeometryType::Hyperbolic | GeometryType::Hyperbolic4D | GeometryType::Hyperbolic5D)
    }

    /// Whether this is a higher-dimensional geometry (4D or 5D).
    pub fn is_higher_dimensional(&self) -> bool {
        matches!(self, GeometryType::Hyperspherical4D | GeometryType::Hyperspherical5D | GeometryType::Hyperbolic4D | GeometryType::Hyperbolic5D)
    }

    /// Get the curvature sign: +1 (spherical), -1 (hyperbolic), 0 (euclidean).
    pub fn curvature_sign(&self) -> f32 {
        if self.is_positive_curvature() { 1.0 }
        else if self.is_negative_curvature() { -1.0 }
        else { 0.0 }
    }
}

impl std::fmt::Display for GeometryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeometryType::Euclidean => write!(f, "Euclidean"),
            GeometryType::Hyperbolic => write!(f, "Hyperbolic"),
            GeometryType::Spherical => write!(f, "Spherical"),
            GeometryType::Hyperspherical4D => write!(f, "4D Hypersphere (S3)"),
            GeometryType::Hyperspherical5D => write!(f, "5D Hypersphere (S4)"),
            GeometryType::Hyperbolic4D => write!(f, "4D Hyperbolic (H4)"),
            GeometryType::Hyperbolic5D => write!(f, "5D Hyperbolic (H5)"),
        }
    }
}