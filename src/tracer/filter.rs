use crate::{Vec2, Float};

pub trait Filter {
    fn radius(&self) -> Float;
    fn inv_radius(&self) -> Float { self.radius().recip() }
    fn eval(&self, px: Vec2) -> Float;
}

/// A simple box filter
pub struct BoxFilter {
    radius: Float,
}

impl BoxFilter {
    /// Constructs a new box filter with `radius`
    pub fn new(radius: i32) -> Self {
        Self { radius: radius as Float }
    }
}

impl Filter for BoxFilter {
    fn radius(&self) -> Float { self.radius }
    fn eval(&self, _px: Vec2) -> Float { 1.0 }
}

/// Triangle filter
pub struct TriangleFilter {
    radius: Float,
}

impl TriangleFilter {
    /// Constructs a new triangle filter with `radius`
    pub fn new(radius: i32) -> Self {
        Self { radius: radius as Float }
    }
}

impl Filter for TriangleFilter {
    fn radius(&self) -> Float { self.radius }
    fn eval(&self, px: Vec2) -> Float {
        let offset = (self.radius - px.abs()).max(Vec2::ZERO);
        offset.x * offset.y
    }
}

/// Gaussian filter
pub struct GaussianFilter {
    radius: Float,
    alpha: Float,
    exp: Float,
}

impl GaussianFilter {
    /// New Gaussian filter with `radius` and falloff coefficient `alpha`
    pub fn new(radius: i32, alpha: Float) -> Self {
        let radius = radius as Float;
        let exp = (-alpha * radius * radius).exp();
        Self {
            radius,
            alpha,
            exp,
        }
    }

    fn gaussian(&self, x: Float) -> Float {
        ((-self.alpha * x * x).exp() - self.exp).max(0.0)
    }
}

impl Filter for GaussianFilter {
    fn radius(&self) -> Float { self.radius }
    fn eval(&self, px: Vec2) -> Float {
        self.gaussian(px.x) * self.gaussian(px.y)
    }
}
