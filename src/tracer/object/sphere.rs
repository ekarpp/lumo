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

    /// Visible area from `xo` forms a cone.
    /// Sample a random direction towards `xo` within the cone.
    /// HOW SPECIFICALLY? disk transform?
    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> Ray {
        /* uvw-orthonormal basis,
         * where w is the direction from xo to origin of this sphere. */
        let uvw = Onb::new(self.origin - xo);

        let dist_light2 = xo.distance_squared(self.origin);

        let z = 1.0 + rand_sq.y * ((1.0 - self.radius * self.radius / dist_light2).sqrt() - 1.0);

        let phi = 2.0 * PI * rand_sq.x;
        let x = phi.cos() * (1.0 - z * z).sqrt();
        let y = phi.sin() * (1.0 - z * z).sqrt();

        let wi = uvw.to_world(DVec3::new(x, y, z));

        Ray::new(xo, wi)
    }
    /* make sphere pdf, area pdf, etc..? */
    /// PDF (w.r.t solid angle) for sampling area of the sphere
    /// that is visible from `xo` (a cone)
    fn sample_towards_pdf(&self, ri: &Ray) -> f64 {
        match self.hit(ri, 0.0, INFINITY) {
            None => 0.0,
            Some(_) => {
                let xo = ri.origin;

                let sin2_theta_max = self.radius * self.radius / self.origin.distance_squared(xo);
                let cos_theta_max = (1.0 - sin2_theta_max).sqrt();

                1.0 / (2.0 * PI * (1.0 - cos_theta_max))
            }
        }
    }
}
