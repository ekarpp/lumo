use crate::{ Float, Direction, Point, Vec3, Mat3, Vec4, Mat4 };
use std::ops::Mul;
use std::fmt;

#[cfg(test)]
mod transform_tests;

// TODO: separate projection from affine
pub struct Transform {
    m: Mat4,
    inv: Mat4,
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\n  \"m\": {},\n  \"inv\": {}\n}}", self.m, self.inv)
    }
}

impl Transform {
    #[inline]
    pub fn row(&self, y: usize) -> &Vec4 {
        match y {
            0 => &self.m.y0,
            1 => &self.m.y1,
            2 => &self.m.y2,
            3 => &self.m.y3,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self {
            m: self.m.abs(),
            inv: Mat4::new(Vec4::NAN, Vec4::NAN, Vec4::NAN, Vec4::NAN),
        }
    }

    #[inline]
    pub fn to_normal(&self) -> Mat3 {
        self.m.to_mat3().inv().transpose()
    }

    #[inline]
    pub fn to_normal_inv(&self) -> Mat3 {
        self.m.to_mat3().transpose()
    }

    #[inline]
    pub fn to_mat3(&self) -> Mat3 {
        self.m.to_mat3()
    }

    #[inline]
    pub fn to_translation(&self) -> Vec3 {
        Vec3::new(
            self.m.y0.w,
            self.m.y1.w,
            self.m.y2.w,
        )
    }

    #[inline]
    pub fn mat3(m3: Mat3) -> Self {
        let inv = Mat4::mat3(m3.inv());
        let m = Mat4::mat3(m3);

        Self { m, inv }
    }

    #[inline]
    pub fn transform_pt(&self, p: Point) -> Point {
        (self.m.mul_vec4(p.extend(1.0))).project()
    }

    #[inline]
    pub fn transform_pt_inv(&self, p: Point) -> Point {
        (self.inv.mul_vec4(p.extend(1.0))).project()
    }

    #[inline]
    pub fn transform_dir(&self, d: Direction) -> Direction {
        (self.m.mul_vec4(d.extend(0.0))).project()
    }

    #[inline]
    pub fn transform_dir_inv(&self, d: Direction) -> Direction {
        (self.inv.mul_vec4(d.extend(0.0))).project()
    }

    pub fn perspective(near: Float, far: Float) -> Self {
        let a = far / (far - near);
        let b = -far * near / (far - near);

        let m = Mat4::new(
            Vec4::X,
            Vec4::Y,
            Vec4::Z * a + Vec4::W * b,
            Vec4::Z,
        );
        let inv = Mat4::new(
            Vec4::X,
            Vec4::Y,
            Vec4::W,
            Vec4::Z * (1.0 / b) + Vec4::W * (1.0 / near),
        );

        Self { m, inv }
    }

    pub fn translation(x: Float, y: Float, z: Float) -> Self {
        let m = Mat4::new(
            Vec4::X + Vec4::W * x,
            Vec4::Y + Vec4::W * y,
            Vec4::Z + Vec4::W * z,
            Vec4::W,
        );
        let inv = Mat4::new(
            Vec4::X + Vec4::W * (-x),
            Vec4::Y + Vec4::W * (-y),
            Vec4::Z + Vec4::W * (-z),
            Vec4::W,

        );

        Self { m, inv }
    }

    pub fn scale(x: Float, y: Float, z: Float) -> Self {
        Self::mat3(Mat3::diag(Vec3::new(x, y, z)))
    }

    pub fn rotate_x(theta: Float) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();

        Self::mat3(Mat3::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, cos, -sin),
            Vec3::new(0.0, sin, cos),
        ))
    }

    pub fn rotate_y(theta: Float) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();

        Self::mat3(Mat3::new(
            Vec3::new(cos, 0.0, sin),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(-sin, 0.0, cos),
        ))
    }

    pub fn rotate_z(theta: Float) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();

        Self::mat3(Mat3::new(
            Vec3::new(cos, -sin, 0.0),
            Vec3::new(sin, cos, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ))
    }

}

impl Mul for Transform {
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: Transform) -> Self::Output {
        Self {
            m: self.m * rhs.m,
            inv: rhs.inv * self.inv,
        }
    }
}
