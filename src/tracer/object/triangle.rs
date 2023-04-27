use super::*;

#[cfg(test)]
mod triangle_tests;

/// Triangle specified by three points
pub struct Triangle {
    /// Point `a`
    a: DVec3,
    /// Point `b`
    b: DVec3,
    /// Point `c`
    c: DVec3,
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
            b,
            c,
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
        AaBoundingBox::new(
            self.a.min(self.b.min(self.c)),
            self.a.max(self.b.max(self.c)),
        )
    }
}

impl Object for Triangle {
    /// Barycentric triangle intersection with Möller-Trumbore algorithm
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let xo = r.origin;

        let wi_abs = r.dir.abs();
        // index for max component, permute it cyclically to z position
        let kz = if wi_abs.x > wi_abs.y {
            if wi_abs.x > wi_abs.z { 0 } else { 2 }
        } else {
            if wi_abs.y > wi_abs.z { 1 } else { 2 }
        };

        let permute = |vec: DVec3| {
            match kz {
                0 => DVec3::new(vec.y, vec.z, vec.x),
                1 => DVec3::new(vec.z, vec.x, vec.y),
                _ => vec,

            }
        };

        // permute to avoid division by zero
        let wi = permute(r.dir);
        let mut at = permute(self.a - xo);
        let mut bt = permute(self.b - xo);
        let mut ct = permute(self.c - xo);

        let shear = DVec3::new(-wi.x, -wi.y, 0.0) / wi.z;

        at += shear * at.z;
        bt += shear * bt.z;
        ct += shear * ct.z;

        let edges = DVec3::new(
            bt.x * ct.y - bt.y * ct.x,
            ct.x * at.y - ct.y * at.x,
            at.x * bt.y - at.y * bt.x,
        );

        if edges.min_element() < 0.0 && edges.max_element() > 0.0 {
            return None;
        }

        let det = edges.dot(DVec3::ONE);

        if det == 0.0 {
            return None;
        }

        let t_scaled = edges.dot(DVec3::new(at.z, bt.z, ct.z)) / wi.z;

        // check that hit is within bounds
        let b1 = det < 0.0 && (t_scaled > t_min * det || t_scaled < t_max * det);
        let b2 = det > 0.0 && (t_scaled < t_min * det || t_scaled > t_max * det);

        if b1 || b2 {
            return None;
        }

        let barycentrics = edges / det;
        let alpha = barycentrics.x;
        let beta = barycentrics.y;
        let gamma = barycentrics.z;
        let t = t_scaled / det;

        let ns = alpha * self.na + beta * self.nb + gamma * self.nc;
        let ns = ns.normalize();

        let uv = alpha * self.ta + beta * self.tb + gamma * self.tc;

        let abc = DMat3::from_cols(self.a, self.b, self.c);

        let err = efloat::gamma(7) * (abc * barycentrics).abs();

        Hit::new(t, &self.material, r.at(t), ns, self.ng, uv)
    }
}

impl Sampleable for Triangle {
    fn area(&self) -> f64 {
        let b_m_a = self.b - self.a;
        let c_m_a = self.c - self.a;
        b_m_a.cross(c_m_a).length() / 2.0
    }

    /// Random point with barycentrics.
    fn sample_on(&self, rand_sq: DVec2) -> (DVec3, DVec3) {
        let gamma = 1.0 - (1.0 - rand_sq.x).sqrt();
        let beta = rand_sq.y * (1.0 - gamma);
        let b_m_a = self.b - self.a;
        let c_m_a = self.c - self.a;

        (self.a + beta * b_m_a + gamma * c_m_a, self.ng)
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
