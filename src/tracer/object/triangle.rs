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
    /// Constructs triangle from three points. Normal determined from order of
    /// the points, they are in counter-clockwise order.
    ///
    /// # Arguments
    /// * `a,b,c` - Three vertices of the triangle
    /// * `material` - Material of the triangle
    pub fn new(abc: (DVec3, DVec3, DVec3), material: Material)
               -> Box<Self> {
        /* check degeneracy */
        let norm = (abc.1 - abc.0).cross(abc.2 - abc.0);
        assert!(norm.length() != 0.0);
        let norm = norm.normalize();
        Box::new(Self {
            a: abc.0,
            b: abc.1,
            c: abc.2,
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
    pub fn new_w_normals(abc: (DVec3, DVec3, DVec3), nabc: (DVec3, DVec3, DVec3))
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

    /// Barycentric triangle intersection with Möller-Trumbore algorithm
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        /* can cache some results on triangle.
         * this faster? https://gamedev.stackexchange.com/questions/23743/whats-the-most-efficient-way-to-find-barycentric-coordinates */
        let e1 = self.b - self.a;
        let e2 = self.c - self.a;
        let pde2 = r.dir.cross(e2);
        let det_a = e1.dot(pde2);
        if det_a.abs() < EPSILON {
            return None;
        }

        let vec_b = r.origin - self.a;

        let beta = vec_b.dot(pde2) / det_a;
        if !(0.0..=1.0).contains(&beta) {
            return None;
        }

        let pbe1 = vec_b.cross(e1);
        let gamma = r.dir.dot(pbe1) / det_a;

        if gamma < 0.0 || gamma + beta > 1.0 {
            return None;
        }
        let t = e2.dot(pbe1) / det_a;

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
