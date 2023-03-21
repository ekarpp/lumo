use super::*;

#[cfg(test)]
mod triangle_tests;

/// Triangle specified by three points
pub struct Triangle {
    a: DVec3,
    b: DVec3,
    c: DVec3,
    /// Unidirectional normal
    na: DVec3,
    nb: DVec3,
    nc: DVec3,
    material: Material,
}

impl Triangle {
    /// Constructs triangle from three points and specified normal direction
    ///
    /// # Arguments
    /// * `a,b,c` - Three vertices of the triangle
    /// * `norm_dir` - Direction where normal should point
    /// * `material` - Material of the triangle
    pub fn new(a: DVec3, b: DVec3, c: DVec3, norm_dir: DVec3, material: Material)
               -> Box<Self> {
        /* check degeneracy */
        assert!((b - a).cross(c - a).length() != 0.0);

        let norm = (b - a).cross(c - a).normalize();
        let norm = if norm.dot(norm_dir) > 0.0 { norm } else { -norm };
        Box::new(Self {
            a,
            b,
            c,
            material,
            na: norm,
            nb: norm,
            nc: norm,
        })
    }

    /// Create triangle with a specified normal at each vertex. Assigns blank
    /// material, kD-tree should store this and have the material.
    ///
    /// # Arguments
    /// * `abc` - Triple of the triangle vertices
    /// * `nabc` - Triple of the normals at the vertices
    pub fn from_obj(abc: (DVec3, DVec3, DVec3), nabc: (DVec3, DVec3, DVec3))
                    -> Self {
        Self {
            a: abc.0,
            b: abc.1,
            c: abc.2,
            na: nabc.0,
            nb: nabc.1,
            nc: nabc.2,
            material: Material::Blank,
        }
    }
}

impl Bounded for Triangle {
    fn bounding_box(&self) -> AaBoundingBox {
        AaBoundingBox::new(
            self.a.min(self.b.min(self.c)),
            self.a.max(self.b.max(self.c)),
        )
    }
}

impl Object for Triangle {
    fn material(&self) -> &Material { &self.material }

    /// Barycentric triangle intersection with Cramer's rule
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
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

        if t < t_min + EPSILON  || t > t_max - EPSILON {
            None
        } else {
            let alpha = 1.0 - beta - gamma;
            // correct order?
            let norm = alpha * self.na + beta * self.nb + gamma * self.nc;

            Hit::new(
                t,
                self,
                r.at(t),
                norm,
            )
        }
    }

    /// Random point with barycentrics.
    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        let gamma = 1.0 - (1.0 - rand_sq.x).sqrt();
        let beta = rand_sq.y * (1.0 - gamma);

        self.a + beta * (self.b - self.a) + gamma * (self.c - self.a)
    }

    /// Choose random point on surface of triangle. Shoot ray towards it.
    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> Ray {
        let xi = self.sample_on(rand_sq);
        let wi = xi - xo;
        Ray::new(xo, wi)
    }

    fn sample_towards_pdf(&self, ri: &Ray) -> f64 {
        match self.hit(ri, 0.0, INFINITY) {
            None => 0.0,
            Some(hi) => {
                let area = (self.b - self.a).cross(self.c - self.a).length()
                    / 2.0;

                let xo = ri.origin;
                let xi = hi.p;
                let ni = hi.norm;
                let wi = ri.dir;
                xo.distance_squared(xi)
                    / (ni.dot(wi.normalize()).abs() * area)

            }
        }
    }

}
