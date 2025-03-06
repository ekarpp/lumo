use crate::{ Float, Vec3 };
use std::ops::{ Add, Sub, Neg, Mul, Div };
use std::fmt;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vec2 {
    pub x: Float,
    pub y: Float,
}

macro_rules! ew_ops {
    ( $( $op_name:ident ),* ) => {
        $(
            #[inline]
            pub fn $op_name(&self) -> Self {
                Self {
                    x: self.x.$op_name(),
                    y: self.y.$op_name(),
                }
            }
        )*
    }
}

macro_rules! pw_ops {
    ( $( $op_name:ident ),* ) => {
        $(
            #[inline]
            pub fn $op_name(&self, other: Self) -> Self {
                Self {
                    x: self.x.$op_name(other.x),
                    y: self.y.$op_name(other.y),
                }
            }
        )*
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    #[inline]
    pub const fn new(x: Float, y: Float) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn splat(v: Float) -> Self {
        Self { x: v, y: v }
    }

    #[inline]
    pub fn extend(&self, z: Float) -> Vec3 {
        Vec3::new(self.x, self.y, z)
    }

    #[inline]
    pub fn dot(&self, other: Self) -> Float {
        self.x * other.x + self.y * other.y
    }

    ew_ops! { abs, fract, floor }
    pw_ops! { min, max }

}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UVec2 {
    pub x: u64,
    pub y: u64,
}

impl fmt::Display for UVec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl UVec2 {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    #[inline]
    pub fn new(x: u64, y: u64) -> Self {
        Self { x, y }
    }

    pw_ops! { min, max }
}

impl Neg for Vec2 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

macro_rules! impl_op {
    ( $( $struct:ident, $elem:ident, $op_trait:ident, $op_fn:ident, $op:ident ),* ) => {
        $(
            impl $op_trait for $struct {
                type Output = Self;

                #[inline]
                fn $op_fn(self, rhs: Self) -> Self::Output {
                    Self {
                        x: self.x.$op(rhs.x),
                        y: self.y.$op(rhs.y),
                    }
                }
            }

            impl $op_trait<$elem> for $struct {
                type Output = $struct;

                #[inline]
                fn $op_fn(self, rhs: $elem) -> Self::Output {
                    Self {
                        x: self.x.$op(rhs),
                        y: self.y.$op(rhs),
                    }
                }
            }

            impl $op_trait<$struct> for $elem {
                type Output = $struct;

                #[inline]
                fn $op_fn(self, rhs: $struct) -> Self::Output {
                    Self::Output {
                        x: self.$op(rhs.x),
                        y: self.$op(rhs.y),
                    }
                }
            }
        )*
    }
}

impl_op! {
    Vec2, Float, Add, add, add,
    Vec2, Float, Sub, sub, sub,
    Vec2, Float, Mul, mul, mul,
    Vec2, Float, Div, div, div,

    UVec2, u64, Add, add, add,
    UVec2, u64, Sub, sub, saturating_sub,
    UVec2, u64, Mul, mul, mul
}
