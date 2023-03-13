use super::*;

#[cfg(test)]
mod triangle_tests;

/* barycentric interpolation ~ different texture at each point of triangle */
/* normal inside?? */

/// Triangle specified by three points
pub struct Triangle {
    a: DVec3,
    b: DVec3,
    c: DVec3,
    norm: DVec3,
    material: Material,
}

impl Triangle {
    /// Constructs triangle from three points and specifies normal direction
    ///
    /// # Arguments
    /// * `a,b,c` - Three vertices of the triangle
    /// * `n` - Direction where normal should point
    /// * `m` - Material of the triangle
    pub fn new(a: DVec3, b: DVec3, c: DVec3, n: DVec3, m: Material)
               -> Box<Self> {
        /* check degeneracy */
        assert!((b - a).cross(c - a).length() != 0.0);

        let norm = (b - a).cross(c - a).normalize();

        Box::new(Self {
            a: a,
            b: b,
            c: c,
            norm: if norm.dot(n) > 0.0 { norm } else { -norm },
            material: m,
        })
    }
}

impl Object for Triangle {
    fn material(&self) -> &Material { &self.material }

    fn normal_at(&self, _p: DVec3) -> DVec3 {
        self.norm
    }

    fn area(&self) -> f64 {
        (self.b - self.a).cross(self.c - self.a).length() / 2.0
    }

    /// Barycentric triangle intersection with Cramer's rule
    fn hit(&self, r: &Ray) -> Option<Hit> {
        /* can store a-c, a-b, and a instead. saves some computation.
         * compiler should do it? */

        let mat_a = DMat3::from_cols(
            self.a - self.b,
            self.a - self.c,
            r.dir
        );

        let det_a = mat_a.determinant();

        if det_a.abs() < EPSILON {
            return None;
        }

        let vec_b = self.a - r.origin;

        let beta = DMat3::from_cols(
            vec_b,
            mat_a.col(1),
            mat_a.col(2),
        ).determinant() / det_a;

        let gamma = DMat3::from_cols(
            mat_a.col(0),
            vec_b,
            mat_a.col(2),
        ).determinant() / det_a;

        if beta < 0.0 || gamma < 0.0
            || beta + gamma > 1.0 {
            return None;
        }

        let t = DMat3::from_cols(
            mat_a.col(0),
            mat_a.col(1),
            vec_b,
        ).determinant() / det_a;

        if t < EPSILON {
            None
        } else {
            Hit::new(
                t,
                self,
                r,
            )
        }
    }

    fn sample_towards(&self, p: DVec3, rand_sq: DVec2) -> DVec3 {
        let gamma = 1.0 - (1.0 - rand_sq.x).sqrt();
        let beta = rand_sq.y * (1.0 - gamma);

        self.a + beta * (self.b - self.a) + gamma * (self.c - self.a) - p
    }
}
