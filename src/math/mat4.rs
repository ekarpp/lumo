use crate::{ Float, Mat3, Vec3 };
use std::ops::{ Add, Mul };
use std::fmt;

#[cfg(test)]
mod mat4_tests;

#[derive(PartialEq, Clone)]
#[repr(C)]
pub struct Vec4 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
    pub w: Float,
}

impl fmt::Display for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}, {}]", self.x, self.y, self.z, self.w)
    }
}

impl Vec4 {
    pub const X: Self = Self { x: 1.0, y: 0.0, z: 0.0, w: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0, z: 0.0, w: 0.0 };
    pub const Z: Self = Self { x: 0.0, y: 0.0, z: 1.0, w: 0.0 };
    pub const W: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    pub const NAN: Self = Self {
        x: Float::NAN,
        y: Float::NAN,
        z: Float::NAN,
        w: Float::NAN
    };

    #[inline]
    pub fn new(x: Float, y: Float, z: Float, w: Float) -> Self {
        Self { x, y, z, w }
    }

    #[inline(always)]
    pub fn truncate(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    #[inline(always)]
    pub fn project(&self) -> Vec3 {
        if self.w == 0.0 {
            self.truncate()
        } else {
            self.truncate() / self.w
        }
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self { x: self.x.abs(), y: self.y.abs(), z: self.z.abs(), w: self.w.abs() }
    }

    #[inline(always)]
    pub fn dot(&self, other: &Self) -> Float {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }
}

impl Add for Vec4 {
    type Output = Vec4;

    #[inline]
    fn add(self, rhs: Vec4) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Mul for Vec4 {
    type Output = Vec4;

    #[inline]
    fn mul(self, rhs: Vec4) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }
}

impl Mul<Float> for Vec4 {
    type Output = Vec4;

    #[inline]
    fn mul(self, rhs: Float) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

#[derive(PartialEq, Clone)]
#[repr(C)]
pub struct Mat4 {
    pub y0: Vec4,
    pub y1: Vec4,
    pub y2: Vec4,
    pub y3: Vec4,
}

impl fmt::Display for Mat4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[\n  {},\n  {},\n  {},\n  {}\n]", self.y0, self.y1, self.y2, self.y3)
    }
}

impl Mat4 {
    #[allow(dead_code)]
    pub const ID: Self = Self { y0: Vec4::X, y1: Vec4::Y, y2: Vec4::Z, y3: Vec4::W, };

    #[inline]
    pub fn new(y0: Vec4, y1: Vec4, y2: Vec4, y3: Vec4) -> Self {
        Self { y0, y1, y2, y3 }
    }

    #[inline]
    pub fn to_mat3(&self) -> Mat3 {
        Mat3::new(
            self.y0.truncate(),
            self.y1.truncate(),
            self.y2.truncate(),
        )
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self {
            y0: self.y0.abs(),
            y1: self.y1.abs(),
            y2: self.y2.abs(),
            y3: self.y3.abs(),
        }
    }

    #[inline]
    pub fn mat3(m3: Mat3) -> Self {
        Self {
            y0: m3.y0.extend(0.0),
            y1: m3.y1.extend(0.0),
            y2: m3.y2.extend(0.0),
            y3: Vec4::W,
        }
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        let y0 = Vec4::new(self.y0.x, self.y1.x, self.y2.x, self.y3.x);
        let y1 = Vec4::new(self.y0.y, self.y1.y, self.y2.y, self.y3.y);
        let y2 = Vec4::new(self.y0.z, self.y1.z, self.y2.z, self.y3.z);
        let y3 = Vec4::new(self.y0.w, self.y1.w, self.y2.w, self.y3.w);

        Self { y0, y1, y2, y3, }
    }

    #[inline]
    pub fn mul_vec4(&self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.y0.dot(&rhs),
            self.y1.dot(&rhs),
            self.y2.dot(&rhs),
            self.y3.dot(&rhs),
        )
    }
}

impl Mul for Mat4 {
    type Output = Mat4;

    #[inline]
    fn mul(self, rhs: Mat4) -> Mat4 {
        let t = rhs.transpose();
        let y0 = Vec4::new(
            self.y0.dot(&t.y0),
            self.y0.dot(&t.y1),
            self.y0.dot(&t.y2),
            self.y0.dot(&t.y3),
        );
        let y1 = Vec4::new(
            self.y1.dot(&t.y0),
            self.y1.dot(&t.y1),
            self.y1.dot(&t.y2),
            self.y1.dot(&t.y3),
        );
        let y2 = Vec4::new(
            self.y2.dot(&t.y0),
            self.y2.dot(&t.y1),
            self.y2.dot(&t.y2),
            self.y2.dot(&t.y3),
        );
        let y3 = Vec4::new(
            self.y3.dot(&t.y0),
            self.y3.dot(&t.y1),
            self.y3.dot(&t.y2),
            self.y3.dot(&t.y3),
        );

        Self { y0, y1, y2, y3 }
    }
}
