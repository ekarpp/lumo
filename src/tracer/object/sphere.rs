use super::*;

#[cfg(test)]
mod sphere_tests;

/// Sphere specified by its radius and origin
pub struct Sphere {
    /// Origin of the sphere
    pub origin: DVec3,
    /// Radius of the sphere
    pub radius: f64,
    /// Material of the sphere
    material: Material,
}

impl Sphere {
    /// # Arguments
    /// * `origin` - Origin of the sphere
    /// * `radius` - Radius of the sphere
    /// * `material` - Material of the sphere
    pub fn new(origin: DVec3, radius: f64, material: Material) -> Box<Self> {
        assert!(radius != 0.0);

        Box::new(Self {
            origin,
            radius,
            material,
        })
    }
}

impl Bounded for Sphere {
    fn bounding_box(&self) -> AaBoundingBox {
        let r_vec = DVec3::splat(self.radius);
        AaBoundingBox::new(self.origin - r_vec, self.origin + r_vec)
    }
}

impl Object for Sphere {
    /// Solve the quadratic
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let xo = r.origin;
        let wi = r.dir;

        let dx = EFloat64::from(wi.x);
        let dy = EFloat64::from(wi.y);
        let dz = EFloat64::from(wi.z);

        let ox = EFloat64::from(xo.x) - EFloat64::from(self.origin.x);
        let oy = EFloat64::from(xo.y) - EFloat64::from(self.origin.y);
        let oz = EFloat64::from(xo.z) - EFloat64::from(self.origin.z);

        let radius2 = EFloat64::from(self.radius) * EFloat64::from(self.radius);

        let a = dx * dx + dy * dy + dz * dz;
        let b = EFloat64::from(2.0) * (dx * ox + dy * oy + dz * oz);
        let c = ox * ox + oy * oy + oz * oz - radius2;

        let t0t1 = EFloat64::quadratic(a, b, c);
        if t0t1.is_none() {
            return None;
        }
        let (t0, t1) = t0t1.unwrap();

        // sphere too far or behind
        if t0.high > t_max || t1.low <= t_min {
            return None;
        }

        let t = if t0.low <= t_min { t1 } else { t0 };
        let xi = r.at(t.value);
        // reproject to sphere to reduce floating point error
        let xi = xi * radius2.value / xi.distance_squared(self.origin);
        let ni = (xi - self.origin) / self.radius;

        let u = ((-ni.z).atan2(ni.x) + PI) / (2.0 * PI);
        let v = (-ni.y).acos() / PI;
        let uv = DVec2::new(u, v);

        let err = efloat::gamma(5) * xi.abs();

        Hit::new(t.value, &self.material, xi, err, ni, ni, uv)
    }
}

impl Sampleable for Sphere {
    fn area(&self) -> f64 {
        4.0 * PI * self.radius * self.radius
    }

    /// Sample on unit sphere and scale
    fn sample_on(&self, rand_sq: DVec2) -> (DVec3, DVec3) {
        let rand_sph = rand_utils::square_to_sphere(rand_sq);

        let xo = self.origin + self.radius * rand_sph;
        let ng = (xo - self.origin) / self.radius;

        (xo, ng)
    }

    /// Visible area from `xo` forms a cone. Sample a random point on the
    /// spherical cap that the visible area forms. Return a ray with direction
    /// towards the sampled point.
    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> DVec3 {
        let dist_origin2 = xo.distance_squared(self.origin);
        let radius2 = self.radius * self.radius;

        let xi = if dist_origin2 < radius2 {
            // if inside sphere, just sample on the surface
            let (xi, _) = self.sample_on(rand_sq);
            xi
        } else {
            /* uvw-orthonormal basis,
             * where w is the direction from xo to origin of this sphere. */
            let uvw = Onb::new(self.origin - xo);

            let dist_origin = dist_origin2.sqrt();

            // theta_max = maximum angle of the visible cone to sphere

            let sin2_theta_max = radius2 / dist_origin2;
            let cos_theta_max = (1.0 - sin2_theta_max).max(0.0).sqrt();

            let cos_theta = (1.0 - rand_sq.x) + rand_sq.x * cos_theta_max;
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
            let phi = 2.0 * PI * rand_sq.y;

            // we have a point on the disk base of the cone.
            // consider disk origin to be at the sphere origin, say `xs`.
            // we compute normal at the point on the sphere where the direction
            // `xs - xo` from `xo` intersects the sphere. then add the normal
            // scaled to radius to the origin of the sphere to get the point
            // on the spherical cap.

            let dist_sampled = dist_origin * cos_theta
                - (radius2 - dist_origin2 * sin_theta * sin_theta).max(0.0).sqrt();

            // alpha = angle between `origin - xo` and normal at sampled point
            let cos_alpha = (dist_origin2 + radius2 - dist_sampled * dist_sampled)
                / (2.0 * dist_origin * self.radius);
            let sin_alpha = (1.0 - cos_alpha * cos_alpha).max(0.0).sqrt();

            let ng_local = DVec3::new(
                phi.cos() * sin_alpha,
                phi.sin() * sin_alpha,
                cos_alpha,
            );

            let ng = uvw.to_world(ng_local);

            self.origin + ng * self.radius
        };

        xi - xo
    }
    /* make sphere pdf, area pdf, etc..? */
    /// PDF (w.r.t area) for sampling area of the sphere
    /// that is visible from `xo` (a spherical cap formed by a cone)
    fn sample_towards_pdf(&self, ri: &Ray) -> (f64, Option<Hit>) {
        match self.hit(ri, 0.0, INFINITY) {
            None => (0.0, None),
            Some(hi) => {
                let xo = ri.origin;

                let radius2 = self.radius * self.radius;
                let dist_origin2 = xo.distance_squared(self.origin);

                let area = if dist_origin2 < radius2 {
                    4.0 * PI * radius2
                } else {
                    /* this computes the area of the spherical cap of the visible
                     * area. slightly faster way is to directly compute the
                     * solid angle of the visible cone. the area is then with
                     * respect to the disk at the base of the spherical cap though
                     */

                    let dist_origin = dist_origin2.sqrt();

                    let sin2_theta_max = radius2 / dist_origin2;
                    let cos_theta_max = (1.0 - sin2_theta_max).max(0.0).sqrt();

                    let dist_tangent = cos_theta_max * dist_origin;

                    let cos_alpha_max =
                        (dist_origin2 + radius2 - dist_tangent * dist_tangent)
                        / (2.0 * dist_origin * self.radius);

                    2.0 * PI * (1.0 - cos_alpha_max) * radius2
                };

                (1.0 / area, Some(hi))
            }
        }
    }
}
