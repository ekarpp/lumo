use crate::{Vec2, Float};

pub trait Filter {
    fn radius(&self) -> Float;
    fn inv_radius(&self) -> Float;
    fn eval(&self, px: Vec2) -> Float;
}

/// A simple box filter
pub struct BoxFilter {
    radius: Float,
}

impl BoxFilter {
    /// Constructs a new box filter with `radius`
    pub fn new(radius: i32) -> Self {
        Self {
            radius: radius as Float,
        }
    }
}

impl Filter for BoxFilter {
    fn radius(&self) -> Float { self.radius }
    fn inv_radius(&self) -> Float { self.radius.recip() }
    fn eval(&self, _px: Vec2) -> Float { 1.0 }
}
