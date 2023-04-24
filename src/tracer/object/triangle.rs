use super::*;

#[cfg(test)]
mod triangle_tests;

/// Triangle specified by three points
pub struct Triangle {
    /// Point `a`
    a: DVec3,
    /// Point `b` minus `a`
    b_m_a: DVec3,
    /// Point `c` minus `a`
    c_m_a: DVec3,
    /// Geometric normal. CCW in the order of vertices.
    ng: DVec3,
    /// Shading normal for the vertex `a`
    na: DVec3,
    /// Shading normal for the vertex `b`
    nb: DVec3,
    /// Shading normal for the vertex `c`
    nc: DVec3,
    /// Texutre coordinate for the vertex `a`
    ta: DVec2,
    /// Texutre coordinate for the vertex `b`
    tb: DVec2,
    /// Texutre coordinate for the vertex `c`
    tc: DVec2,
    /// Material of the triangle
    material: Material,
}

impl Triangle {
    /// Constructs triangle from three points. Normal determined from order of
    /// the points, they are in counter-clockwise order.
    ///
    /// # Arguments
    /// * `abc` - Three vertices of the triangle stored in the columns
    /// * `nabc` - Optional shading normals for each vertex stored in the columns
    /// * `tabc` - Optional texture coordinates for each vertex stored in the columns
    /// * `material` - Material of the triangle
    pub fn new(
        abc: DMat3,
        nabc: Option<DMat3>,
        tabc: Option<DMat3>,
        material: Material,
    ) -> Box<Self> {
        let a = abc.col(0);
        let b = abc.col(1);
        let c = abc.col(2);
        /* check degeneracy */
        let b_m_a = b - a;
        let c_m_a = c - a;
        let ng = (b_m_a).cross(c_m_a);
        assert!(ng.length() != 0.0);
        let ng = ng.normalize();

        let (na, nb, nc) = match nabc {
            None => (ng, ng, ng),
            Some(nabc) => (nabc.col(0), nabc.col(1), nabc.col(2)),
        };

        let (ta, tb, tc) = match tabc {
            None => (DVec2::ZERO, DVec2::X, DVec2::ONE),
            Some(tabc) => (
                tabc.col(0).truncate(),
                tabc.col(1).truncate(),
                tabc.col(2).truncate(),
            ),
        };

        Box::new(Self {
            a,
            b_m_a,
            c_m_a,
            material,
            ng,
            na,
            nb,
            nc,
            ta,
            tb,
            tc,
        })
    }
}

impl Bounded for Triangle {
    fn bounding_box(&self) -> AaBoundingBox {
        // something better can be done?
        let b = self.b_m_a + self.a;
        let c = self.c_m_a + self.a;
        AaBoundingBox::new(
            self.a.min(b.min(c)),
            self.a.max(b.max(c)),
        )
    }
}

impl Object for Triangle {
    /// Barycentric triangle intersection with Möller-Trumbore algorithm
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        /* can cache some results on triangle. */
        let pde2 = r.dir.cross(self.c_m_a);
        let det_a = self.b_m_a.dot(pde2);
        if det_a.abs() < EPSILON {
            return None;
        }

        let vec_b = r.origin - self.a;

        let beta = vec_b.dot(pde2) / det_a;
        if !(0.0..=1.0).contains(&beta) {
            return None;
        }

        let pbe1 = vec_b.cross(self.b_m_a);
        let gamma = r.dir.dot(pbe1) / det_a;

        if gamma < 0.0 || gamma + beta > 1.0 {
            return None;
        }
        let t = self.c_m_a.dot(pbe1) / det_a;

        if t < t_min + EPSILON || t > t_max {
            None
        } else {
            let alpha = 1.0 - beta - gamma;

            let ns = alpha * self.na + beta * self.nb + gamma * self.nc;
            let ns = ns.normalize();

            let uv = alpha * self.ta + beta * self.tb + gamma * self.tc;

            Hit::new(t, &self.material, r.at(t), ns, self.ng, uv)
        }
    }
}

impl Sampleable for Triangle {
    fn area(&self) -> f64 {
        self.b_m_a.cross(self.c_m_a).length() / 2.0
    }

    /// Random point with barycentrics.
    fn sample_on(&self, rand_sq: DVec2) -> (DVec3, DVec3) {
        let gamma = 1.0 - (1.0 - rand_sq.x).sqrt();
        let beta = rand_sq.y * (1.0 - gamma);

        (self.a + beta * self.b_m_a + gamma * self.c_m_a, self.ng)
    }

    /// Choose random point on surface of triangle. Shoot ray towards it.
    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> DVec3 {
        let (xi, _) = self.sample_on(rand_sq);
        xi - xo
    }

    fn sample_towards_pdf(&self, ri: &Ray) -> (f64, Option<Hit>) {
        match self.hit(ri, 0.0, INFINITY) {
            None => (0.0, None),
            Some(hi) => {
                let p = 1.0 / self.area();

                (p, Some(hi))
            }
        }
    }
}
