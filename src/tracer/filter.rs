use crate::{Vec2, Float};

pub trait Filter {
    fn radius(&self) -> Float;
    fn inv_radius(&self) -> Float;
    fn eval(&self, px: Vec2) -> Float;
}

pub struct BoxFilter {

}

impl BoxFilter {
    pub fn new() -> Self { Self { } }
}

impl Filter for BoxFilter {
    fn radius(&self) -> Float { 1.0 }
    fn inv_radius(&self) -> Float { 1.0 }
    fn eval(&self, _px: Vec2) -> Float { 1.0 }
}
