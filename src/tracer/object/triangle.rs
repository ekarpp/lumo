use super::*;

#[cfg(test)]
mod triangle_tests;

/* barycentric interpolation ~ different texture at each point of triangle */
/* normal inside?? */
pub struct Triangle {
    a: DVec3,
    b: DVec3,
    c: DVec3,
    norm: DVec3,
    material: Material,
}

impl Triangle {
    /* assume non-degenerate */
    pub fn new(a: DVec3, b: DVec3, c: DVec3, m: Material) -> Box<Self> {
        let norm = -(b - a).cross(c - a).normalize();

        Box::new(Self {
            a: a,
            b: b,
            c: c,
            norm: norm,
            material: m,
        })
    }
}

impl Object for Triangle {
    fn material(&self) -> &Material { &self.material }

    fn normal_for_at(&self, r: &Ray, _p: DVec3) -> DVec3 {
        _orient_normal(self.norm, r)
    }

    fn area(&self) -> f64 {
        (self.b - self.a).cross(self.c - self.a).length() / 2.0
    }

    /* barycentric triangle intersection with Cramer's rule */
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

    fn sample_ray(&self, h: &Hit, rand_sq: DVec2) -> Ray {
        let gamma = 1.0 - (1.0 - rand_sq.x).sqrt();
        let beta = rand_sq.y * (1.0 - gamma);

        Ray::new(
            h.p,
            self.a + beta * (self.b - self.a) + gamma * (self.c - self.a) - h.p,
            0,
        )
    }
}
