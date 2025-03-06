use crate::{ Float, Vec3 };
use std::ops::Mul;
use std::fmt;

#[cfg(test)]
mod mat3_tests;

/// Row major 3x3 matrix
#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct Mat3 {
    pub y0: Vec3,
    pub y1: Vec3,
    pub y2: Vec3,
}

impl fmt::Display for Mat3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[\n  {},\n  {},\n  {}\n]", self.y0, self.y1, self.y2)
    }
}

impl Mat3 {
    pub const ID: Self = Self { y0: Vec3::X, y1: Vec3::Y, y2: Vec3::Z };

    #[inline]
    pub fn new(y0: Vec3, y1: Vec3, y2: Vec3) -> Self {
        Self { y0, y1, y2, }
    }

    #[inline]
    pub fn diag(d: Vec3) -> Self {
        Self::new(
            Vec3::X * d.x,
            Vec3::Y * d.y,
            Vec3::Z * d.z,
        )
    }

    #[inline]
    pub fn det(&self) -> Float {
        // sarrus
        let pos = self.y0.x * self.y1.y * self.y2.z
            + self.y0.y * self.y1.z * self.y2.x
            + self.y0.z * self.y1.x * self.y2.y;

        let neg = self.y0.z * self.y1.y * self.y2.x
            + self.y0.y * self.y1.x * self.y2.z
            + self.y0.x * self.y1.z * self.y2.y;

        pos - neg
    }

    #[inline]
    pub fn transpose(&self) -> Mat3 {
        Self::new(
            Vec3::new(self.y0.x, self.y1.x, self.y2.x),
            Vec3::new(self.y0.y, self.y1.y, self.y2.y),
            Vec3::new(self.y0.z, self.y1.z, self.y2.z),
        )
    }

    #[inline]
    pub fn inv(&self) -> Mat3 {
        let det = self.det();

        let a = Self::new(
            self.y1.cross(self.y2),
            self.y2.cross(self.y0),
            self.y0.cross(self.y1)
        ).transpose();

        a * (1.0 / det)
    }

    #[inline]
    pub fn mul_vec3(&self, rhs: Vec3) -> Vec3 {
        Vec3::new(
            self.y0.dot(rhs),
            self.y1.dot(rhs),
            self.y2.dot(rhs),
        )
    }
}

impl Mul<Float> for Mat3 {
    type Output = Mat3;

    #[inline]
    fn mul(self, rhs: Float) -> Mat3 {
        Self::new(
            self.y0 * rhs,
            self.y1 * rhs,
            self.y2 * rhs,
        )
    }
}

impl Mul<Mat3> for Float {
    type Output = Mat3;

    #[inline]
    fn mul(self, rhs: Mat3) -> Mat3 {
        Mat3::new(
            self * rhs.y0,
            self * rhs.y1,
            self * rhs.y2,
        )
    }
}

impl Mul for Mat3 {
    type Output = Mat3;

    #[inline]
    fn mul(self, rhs: Mat3) -> Mat3 {
        let t = rhs.transpose();
        let y0 = Vec3::new(
            self.y0.dot(t.y0),
            self.y0.dot(t.y1),
            self.y0.dot(t.y2),
        );
        let y1 = Vec3::new(
            self.y1.dot(t.y0),
            self.y1.dot(t.y1),
            self.y1.dot(t.y2),
        );
        let y2 = Vec3::new(
            self.y2.dot(t.y0),
            self.y2.dot(t.y1),
            self.y2.dot(t.y2),
        );
        Self::new(y0, y1, y2)
    }
}
