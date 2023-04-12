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
    /// Shading normal for the vertex `a`
    nb: DVec3,
    /// Shading normal for the vertex `a`
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
    pub fn new(abc: (DVec3, DVec3, DVec3), material: Material) -> Box<Self> {
        /* check degeneracy */
        let b_m_a = abc.1 - abc.0;
        let c_m_a = abc.2 - abc.0;
        let ng = (b_m_a).cross(c_m_a);
        assert!(ng.length() != 0.0);
        let ng = ng.normalize();
        Box::new(Self {
            a: abc.0,
            b_m_a,
            c_m_a,
            material,
            ng,
            na: ng,
            nb: ng,
            nc: ng,
        })
    }

    /// Create triangle with a specified normal at each vertex. Assigns blank
    /// material, kD-tree should store this and have the material.
    ///
    /// # Arguments
    /// * `abc` - Triple of the triangle vertices
    /// * `nabc` - Triple of the normals at the vertices
    pub fn new_w_normals(abc: (DVec3, DVec3, DVec3), nabc: (DVec3, DVec3, DVec3)) -> Self {
        let ng = (abc.1 - abc.0).cross(abc.2 - abc.0);
        let ng = if ng.length() == 0.0 {
            #[cfg(debug_assertions)]
            println!("Found degenerate triangle. {:?}", abc);
            // bad .obj file..
            DVec3::Z
        } else {
            ng.normalize()
        };

        Self {
            a: abc.0,
            b_m_a: abc.1 - abc.0,
            c_m_a: abc.2 - abc.0,
            ng,
            na: nabc.0,
            nb: nabc.1,
            nc: nabc.2,
            material: Material::Blank,
        }
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
    fn material(&self) -> &Material {
        &self.material
    }

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

            Hit::new(t, self, r.at(t), ns, self.ng)
        }
    }
}

impl Sampleable for Triangle {
    /// Random point with barycentrics.
    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        let gamma = 1.0 - (1.0 - rand_sq.x).sqrt();
        let beta = rand_sq.y * (1.0 - gamma);

        self.a + beta * self.b_m_a + gamma * self.c_m_a
    }

    /// Choose random point on surface of triangle. Shoot ray towards it.
    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> Ray {
        let xi = self.sample_on(rand_sq);
        let wi = xi - xo;
        Ray::new(xo, wi)
    }

    fn sample_towards_pdf(&self, ri: &Ray) -> (f64, DVec3) {
        match self.hit(ri, 0.0, INFINITY) {
            None => (0.0, DVec3::NAN),
            Some(hi) => {
                let area = self.b_m_a.cross(self.c_m_a).length() / 2.0;

                let xo = ri.origin;
                let xi = hi.p;
                let ni = hi.ng;
                let wi = ri.dir;
                let p = xo.distance_squared(xi) / (ni.dot(wi).abs() * area);

                (p, ni)
            }
        }
    }
}
