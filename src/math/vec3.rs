use crate::{ Axis, Float, Vec2, Vec4 };
use std::fmt;
use std::ops::{
    Add, AddAssign, Sub, Mul, Div, Neg
};

#[cfg(test)]
mod vec3_tests;

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vec3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

macro_rules! ew_ops {
    ( $( $op_name:ident ),* ) => {
        $(
            #[inline(always)]
            pub fn $op_name(&self) -> Self {
                Self {
                    x: self.x.$op_name(),
                    y: self.y.$op_name(),
                    z: self.z.$op_name(),
                }
            }
        )*
    }
}

macro_rules! pw_ops {
    ( $( $op_name:ident ),* ) => {
        $(
            #[inline(always)]
            pub fn $op_name(&self, other: Self) -> Self {
                Self {
                    x: self.x.$op_name(other.x),
                    y: self.y.$op_name(other.y),
                    z: self.z.$op_name(other.z),
                }
            }
        )*
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0, z: 0.0};
    pub const Y: Self = Self { x: 0.0, y: 1.0, z: 0.0};
    pub const Z: Self = Self { x: 0.0, y: 0.0, z: 1.0};
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };

    #[inline]
    pub const fn new(x: Float, y: Float, z: Float) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn splat(v: Float) -> Self {
        Self { x: v, y: v, z: v }
    }

    #[inline]
    pub fn truncate(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    #[inline(always)]
    pub fn extend(&self, w: Float) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }

    #[inline]
    pub fn axis(&self, axis: Axis) -> Float {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    #[inline]
    pub fn length(&self) -> Float {
        self.length_squared().max(0.0).sqrt()
    }

    #[inline]
    pub fn length_squared(&self) -> Float {
        self.dot(*self)
    }

    #[inline]
    pub fn distance(&self, other: Self) -> Float {
        self.distance_squared(other).max(0.0).sqrt()
    }

    #[inline]
    pub fn distance_squared(&self, other: Self) -> Float {
        (*self - other).length_squared()
    }

    #[inline]
    pub const fn scale(&self, s: Float) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    #[inline]
    pub const fn cross(&self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    #[inline(always)]
    pub const fn dot(&self, rhs: Self) -> Float {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline]
    pub fn project_onto(&self, n: Vec3) -> Self {
        n * self.dot(n) / n.length_squared()
    }

    #[inline(always)]
    pub fn normalize(&self) -> Self {
        *self / self.length()
    }

    #[inline]
    pub fn is_normalized(&self) -> bool {
        (self.length_squared() - 1.0).abs() < crate::EPSILON
    }

    ew_ops! { abs, floor, fract }
    pw_ops! { min, max }

    #[inline(always)]
    pub fn min_element(&self) -> Float {
        self.x.min(self.y.min(self.z))
    }

    #[inline(always)]
    pub fn max_element(&self) -> Float {
        self.x.max(self.y.max(self.z))
    }
}

impl Neg for Vec3 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl AddAssign for Vec3 {

    #[inline]
    fn add_assign(&mut self, rhs: Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

macro_rules! impl_op {
    ( $( $op_trait:ident, $op_fn:ident, $op:tt ),* ) => {
        $(
            impl $op_trait for Vec3 {
                type Output = Self;

                #[inline]
                fn $op_fn(self, rhs: Self) -> Self::Output {
                    Self {
                        x: self.x $op rhs.x,
                        y: self.y $op rhs.y,
                        z: self.z $op rhs.z,
                    }
                }
            }

            impl $op_trait<Float> for Vec3 {
                type Output = Vec3;

                #[inline]
                fn $op_fn(self, rhs: Float) -> Self::Output {
                    Self {
                        x: self.x $op rhs,
                        y: self.y $op rhs,
                        z: self.z $op rhs,
                    }
                }
            }

            impl $op_trait<Vec3> for Float {
                type Output = Vec3;

                #[inline]
                fn $op_fn(self, rhs: Vec3) -> Self::Output {
                    Vec3 {
                        x: self $op rhs.x,
                        y: self $op rhs.y,
                        z: self $op rhs.z,
                    }
                }
            }
        )*
    }
}

impl_op! {
    Add, add, +,
    Sub, sub, -,
    Mul, mul, *,
    Div, div, /
}
