use crate::{Vec2, Float};

pub trait Filter {
    fn eval(&self, px: Vec2) -> Float;
}

pub struct BoxFilter {

}

impl BoxFilter {
    pub fn new() -> Self { Self { } }
}

impl Filter for BoxFilter {
    fn eval(&self, px: Vec2) -> Float {
        1.0
    }
}
