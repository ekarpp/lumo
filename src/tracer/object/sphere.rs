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
    fn area(&self) -> f64 { 4.0 * PI * self.radius * self.radius }

    fn material(&self) -> &Material { &self.material }

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

        let xi = r.at(t);
        let ni = (xi - self.origin) / self.radius;

        Hit::new(
            t,
            self,
            xi,
            ni,
        )
    }

    /// If distance to origin smaller than radius, must be inside
    fn inside(&self, p: DVec3) -> bool {
        self.origin.distance_squared(p)
            < self.radius*self.radius
    }


    /// Sample on unit sphere and scale
    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        let rand_sph = RandomShape::gen_3d(RandomShape::Sphere(rand_sq));

        self.origin + self.radius * rand_sph
    }

    /// Visible area from `xo` forms a cone.
    /// Sample a random direction towards `xo` within the cone.
    /// HOW SPECIFICALLY? disk transform?
    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> Ray {
        /* uvw-orthonormal basis,
         * where w is the direction from xo to origin of this sphere. */
        let uvw = Onb::new(self.origin - xo);

        /* TODO: this seems broken */

        let dist_light = xo.distance_squared(self.origin);

        let z = 1.0 + rand_sq.y *
            ((1.0 - self.radius * self.radius / dist_light).sqrt() - 1.0);

        let phi = 2.0 * PI * rand_sq.x;
        let x = phi.cos() * (1.0 - z*z).sqrt();
        let y = phi.sin() * (1.0 - z*z).sqrt();

        let wi = uvw.to_uvw_basis(DVec3::new(x, y, z));

        Ray::new(xo, wi)
    }

    /// PDF for sampling area of the sphere that is visible from `xo`
    fn sample_towards_pdf(&self, ri: &Ray) -> f64 {
        match self.hit(ri) {
            None => 0.0,
            Some(hi) => {
                let xo = ri.origin;
                let wi = ri.dir;

                let sin2_theta_max = self.radius * self.radius
                    / self.origin.distance_squared(xo);
                let cos_theta_max = (1.0 - sin2_theta_max).sqrt();

                let visible_area = 2.0 * PI * (1.0 - cos_theta_max);

                let ni = hi.norm;
                let xi = hi.p;

                xo.distance_squared(xi)
                    / visible_area * ni.dot(wi.normalize()).abs()
            }
        }
    }
}
