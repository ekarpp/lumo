use super::*;

/// Triangle specified by three points
pub struct Triangle {
    /// Reference to the mesh
    mesh: Arc<TriangleMesh>,
    /// Indices of the vertices in the mesh
    vidx: (usize, usize, usize),
    /// Indices of the shading normals in the mesh
    nidx: Option<(usize, usize, usize)>,
    /// Indices of the texture coordinates in the mesh
    tidx: Option<(usize, usize, usize)>,
}

impl Triangle {
    /// Constructs on triangle of the mesh.
    ///
    /// # Arguments
    /// * `mesh` - Reference to the mesh
    /// * `vidx` - Indices to triangle vertices
    /// * `nidx` - Indices to shading normals
    /// * `tidx` - Indices to texture coordinates
    pub fn new(
        mesh: Arc<TriangleMesh>,
        vidx: (usize, usize, usize),
        nidx: Option<(usize, usize, usize)>,
        tidx: Option<(usize, usize, usize)>,
    ) -> Self {
        Self {
            mesh,
            vidx,
            nidx,
            tidx,
        }
    }

    fn a(&self) -> Point { self.mesh.vertices[self.vidx.0] }
    fn b(&self) -> Point { self.mesh.vertices[self.vidx.1] }
    fn c(&self) -> Point { self.mesh.vertices[self.vidx.2] }

    fn shading_normal(&self, barycentrics: Vec3, ng: Normal) -> Normal {
        match self.nidx {
            None => ng,
            Some(nidx) => {
                let na = self.mesh.normals[nidx.0];
                let nb = self.mesh.normals[nidx.1];
                let nc = self.mesh.normals[nidx.2];
                barycentrics.x * na
                    + barycentrics.y * nb
                    + barycentrics.z * nc
            }
        }
    }

    /// Watertight intersection due to Woop et. al. 2013
    fn _hit<const GEO: bool>(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        let xo = r.origin;

        let wi_abs = r.dir.abs();
        // index for max component, permute it cyclically to z position
        let kz = if wi_abs.x > wi_abs.y && wi_abs.x > wi_abs.z {
            0
        } else if wi_abs.y > wi_abs.z {
            1
        } else {
            2
        };

        let permute = |vec: Vec3| {
            match kz {
                0 => Vec3::new(vec.y, vec.z, vec.x),
                1 => Vec3::new(vec.z, vec.x, vec.y),
                _ => vec,

            }
        };

        // permute to avoid division by zero
        let wi = permute(r.dir);
        let mut at = permute(self.a() - xo);
        let mut bt = permute(self.b() - xo);
        let mut ct = permute(self.c() - xo);

        let shear = Vec3::new(-wi.x, -wi.y, 0.0) / wi.z;

        at += shear * at.z;
        bt += shear * bt.z;
        ct += shear * ct.z;

        let edges = Vec3::new(
            bt.x * ct.y - bt.y * ct.x,
            ct.x * at.y - ct.y * at.x,
            at.x * bt.y - at.y * bt.x,
        );

        if edges.min_element() < 0.0 && edges.max_element() > 0.0 {
            return None;
        }

        let det = edges.dot(Vec3::ONE);

        // ray coplanar to triangle
        if det == 0.0 {
            return None;
        }

        // divide by wi.z here due to the way we apply shear
        let t_scaled = edges.dot(Vec3::new(at.z, bt.z, ct.z)) / wi.z;

        // check that hit is within bounds
        let b1 = det < 0.0 &&
            (t_scaled > t_min * det || t_scaled < t_max * det);
        let b2 = det > 0.0 &&
            (t_scaled < t_min * det || t_scaled > t_max * det);

        if b1 || b2 {
            return None;
        }

        let t = t_scaled / det;

        if !GEO {
            return Hit::from_t(t);
        }

        // compute floating point error and verify we are not below t_min
        let max_z_v = at.z.abs().max(bt.z.abs()).max(ct.z.abs());
        let delta_z = efloat::gamma(3) * max_z_v;

        let max_y_v = at.y.abs().max(bt.y.abs()).max(ct.y.abs());
        let delta_y = efloat::gamma(5) * (max_y_v + max_z_v);

        let max_x_v = at.x.abs().max(bt.x.abs()).max(ct.x.abs());
        let delta_x = efloat::gamma(5) * (max_x_v + max_z_v);

        let delta_e = 2.0 * (efloat::gamma(2) * max_x_v * max_y_v
                                + delta_y * max_x_v + delta_x * max_y_v);

        let max_e = edges.x.abs().max(edges.y.abs()).max(edges.z.abs());

        let delta_t = 3.0 * (efloat::gamma(3) * max_e * max_z_v
                             + delta_e * max_z_v + delta_z * max_e) / det.abs();

        if t <= t_min + delta_t {
            return None;
        }

        let barycentrics = edges / det;
        let alpha = barycentrics.x;
        let beta = barycentrics.y;
        let gamma = barycentrics.z;

        let ng = (self.b() - self.a()).cross(self.c() - self.a()).normalize();
        let ns = self.shading_normal(barycentrics, ng);
        let xi = alpha * self.a() + beta * self.b() + gamma * self.c();

        let (ta, tb, tc) = if let Some(tidx) = self.tidx {
            let ta = self.mesh.uvs[tidx.0];
            let tb = self.mesh.uvs[tidx.1];
            let tc = self.mesh.uvs[tidx.2];
            (ta, tb, tc)
        } else {
            (Vec2::ZERO, Vec2::X, Vec2::ONE)
        };

        let uv = alpha * ta + beta * tb + gamma * tc;

        let err = efloat::gamma(7) * Vec3::new(
            (barycentrics * Vec3::new(self.a().x, self.b().x, self.c().x))
             .abs().dot(Vec3::ONE),
            (barycentrics * Vec3::new(self.a().y, self.b().y, self.c().y))
             .abs().dot(Vec3::ONE),
            (barycentrics * Vec3::new(self.a().z, self.b().z, self.c().z))
             .abs().dot(Vec3::ONE),
        );

        // material will be set by parent object
        Hit::new(t, &Material::Blank, r.dir, xi, err, ns, ng, uv)
    }
}

impl Bounded for Triangle {
    fn bounding_box(&self) -> AaBoundingBox {
        AaBoundingBox::new(
            self.a().min(self.b().min(self.c())),
            self.a().max(self.b().max(self.c())),
        )
    }
}

impl Object for Triangle {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        self._hit::<true>(r, t_min, t_max)
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        self._hit::<false>(r, t_min, t_max).map_or(crate::INF, |h| h.t)
    }
}

impl Sampleable for Triangle {
    fn area(&self) -> Float {
        (self.b() - self.a()).cross(self.c() - self.a()).length() / 2.0
    }

    /// Random point with barycentrics.
    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        let gamma = 1.0 - (1.0 - rand_sq.x).sqrt();
        let beta = rand_sq.y * (1.0 - gamma);
        let alpha = 1.0 - gamma - beta;
        let barycentrics = Vec3::new(alpha, beta, gamma);

        let b_m_a = self.b() - self.a();
        let c_m_a = self.c() - self.a();
        let ng = b_m_a.cross(c_m_a).normalize();
        let ns = self.shading_normal(barycentrics, ng);

        let xo = self.a() + beta * b_m_a + gamma * c_m_a;

        let xo_abs = self.a().abs() + (beta * b_m_a).abs() + (gamma * c_m_a).abs();
        let err = efloat::gamma(6) * xo_abs;

        Hit::new(
            0.0,
            // set by parent
            &Material::Blank,
            -ng,
            xo,
            err,
            ns,
            ng,
            Vec2::ZERO,
        ).unwrap()
    }
}

#[cfg(test)]
mod triangle_tests {
    use super::*;
    fn mesh() -> TriangleMesh {
        TriangleMesh {
            vertices: vec![
                Point::NEG_Y + Point::X,
                Point::Y + Point::X,
                Point::Y + Point::NEG_X,
            ],
            normals: vec![],
            uvs: vec![],
        }
    }

    test_util::test_sampleable!(Triangle::new(Arc::new(mesh()), (0, 1, 2), None, None));
}
