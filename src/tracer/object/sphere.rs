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
    fn material(&self) -> &Material {
        &self.material
    }

    /// Solve the quadratic
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let tmp = r.origin - self.origin;
        // coefficients of "hit quadratic"
        // .dot faster than .length_squared, recheck
        let a = r.dir.dot(r.dir);
        let half_b = tmp.dot(r.dir);
        let c = tmp.dot(tmp) - self.radius * self.radius;
        let disc = half_b * half_b - a * c;

        if disc < 0.0 {
            return None;
        }
        let disc_root = disc.sqrt();
        let mut t = (-half_b - disc_root) / a;
        if t < t_min + EPSILON || t > t_max {
            t = (-half_b + disc_root) / a;
            if t < t_min + EPSILON || t > t_max {
                return None;
            }
        }

        let xi = r.at(t);
        let ni = (xi - self.origin) / self.radius;

        Hit::new(t, self, xi, ni, ni)
    }
}

impl Sampleable for Sphere {
    /// Sample on unit sphere and scale
    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        let rand_sph = rand_utils::square_to_sphere(rand_sq);

        self.origin + self.radius * rand_sph
    }

    /// Visible area from `xo` forms a cone. Sample a random point on the
    /// spherical cap that the visible area forms. Return a ray with direction
    /// towards the sampled point. TODO: `xo` inside sphere
    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> Ray {
        /* uvw-orthonormal basis,
         * where w is the direction from xo to origin of this sphere. */
        let uvw = Onb::new(self.origin - xo);

        // SAMPLING INSIDE SPHERE?

        let dist_origin = xo.distance(self.origin);
        let dist_origin2 = dist_origin * dist_origin;
        let radius2 = self.radius * self.radius;

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

        let xi = self.origin + ng * self.radius;

        let wi = xi - xo;

        Ray::new(xo, wi)
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
                let dist_origin = self.origin.distance(xo);
                let dist_origin2 = dist_origin * dist_origin;

                let sin2_theta_max = radius2 / dist_origin2;
                let cos_theta_max = (1.0 - sin2_theta_max).max(0.0).sqrt();

                let dist_tangent = cos_theta_max * dist_origin;

                let cos_alpha_max =
                    (dist_origin2 + radius2 - dist_tangent * dist_tangent)
                    / (2.0 * dist_origin * self.radius);

                let cap_area = 2.0 * PI * (1.0 - cos_alpha_max) * radius2;

                let p = 1.0 / cap_area;

                // slightly faster way is to directly compute the solid angle
                // of the visible cone. the area is then with respect to the
                // disk at the base of the spherical cap though.

                (p, Some(hi))
            }
        }
    }
}
