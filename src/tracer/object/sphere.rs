use super::*;

#[cfg(test)]
mod sphere_tests;

/// Sphere specified by its radius and origin
pub struct Sphere {
    pub origin: DVec3,
    pub radius: f64,
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

impl Object for Sphere {

    /// If distance to origin smaller than radius, must be inside
    fn inside(&self, r: &Ray) -> bool {
        self.origin.distance_squared(r.origin + EPSILON*r.dir)
            < self.radius*self.radius
    }

    fn area(&self) -> f64 { 4.0 * PI * self.radius * self.radius }

    fn material(&self) -> &Material { &self.material }

    fn normal_at(&self, p: DVec3) -> DVec3 {
        (p - self.origin) / self.radius
    }

    /// Solve the quadratic
    fn hit(&self, r: &Ray) -> Option<Hit> {
        let tmp = r.origin - self.origin;
        // coefficients of "hit quadratic"
        // .dot faster than .length_squared, recheck
        let a = r.dir.dot(r.dir);
        let half_b = tmp.dot(r.dir);
        let c = tmp.dot(tmp) - self.radius*self.radius;
        let disc = half_b*half_b - a*c;

        if disc < 0.0 {
            return None;
        }
        let disc_root = disc.sqrt();
        let mut t = (-half_b - disc_root) / a;
        if t < EPSILON {
            t = (-half_b + disc_root) / a;
            if t < EPSILON {
                return None;
            }
        }
        Hit::new(
            t,
            self,
            r,
        )
    }

    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        let rand_sph = rand_utils::unit_square_to_unit_sphere(rand_sq);

        self.origin + self.radius * rand_sph
    }

    /// Visible area from `ho.p = xo` forms a cone.
    /// Sample a random direction within the cone.
    fn sample_towards(&self, ho: &Hit, rand_sq: DVec2) -> (Ray, f64) {
        let xo = ho.p;
        /* uvw-orthonormal basis,
         * where w is the direction from xo to origin of this sphere. */
        let uvw = Onb::new(self.origin - xo);

        let dist_light = xo.distance_squared(self.origin);

        let z = 1.0 + rand_sq.y *
            ((1.0 - self.radius * self.radius / dist_light).sqrt() - 1.0);

        let phi = 2.0 * PI * rand_sq.x;
        let x = phi.cos() * (1.0 - z*z).sqrt();
        let y = phi.sin() * (1.0 - z*z).sqrt();

        let wi = uvw.to_uvw_basis(DVec3::new(x, y, z));

        // IS dir + h.p CORRECT POINT ON "TOWARDS" OBJECT??
        let xi = wi + xo;

        (
            Ray::new(xo, wi),
            self.sample_towards_pdf(xo, xi, wi, self.normal_at(xi)),
        )
    }

    fn sample_towards_pdf(&self, xo: DVec3, _xi: DVec3, _wi: DVec3, _ni: DVec3)
                          -> f64 {
        let sin2_theta_max = self.radius * self.radius
            / self.origin.distance_squared(xo);
        let cos_theta_max = (1.0 - sin2_theta_max).sqrt();

        1.0 / (2.0 * PI * (1.0 - cos_theta_max))
    }
}
