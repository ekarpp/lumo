use super::*;

#[cfg(test)]
mod sphere_tests;

pub struct Sphere {
    pub origin: DVec3,
    pub radius: f64,
    material: Material,
}

impl Sphere {
    /* assume r != 0 */
    pub fn new(origin: DVec3, r: f64, mat: Material) -> Box<Self> {
        Box::new(Self {
            origin: origin,
            radius: r,
            material: mat,
        })
    }
}

impl Object for Sphere {
    fn inside(&self, r: &Ray) -> bool {
        self.origin.distance_squared(r.origin + EPSILON*r.dir)
            < self.radius*self.radius
    }

    fn area(&self) -> f64 { 4.0 * PI * self.radius * self.radius }

    fn material(&self) -> &Material { &self.material }

    fn normal_for_at(&self, r: &Ray, p: DVec3) -> DVec3 {
        _orient_normal((p - self.origin) / self.radius, r)
    }

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

    /* sample random ray in cone from h.p towards self */
    fn sample_shadow_ray(&self, h: &Hit, rand_sq: DVec2) -> Ray {
        /* uvw-basis orthonormal basis,
         * where w is the direction from x to origin of this sphere. */
        let w = (self.origin - h.p).normalize();
        let (u, v) = _uvw_basis(w);

        let dist_light = h.p.distance_squared(self.origin);

        let z = 1.0 + rand_sq.y *
            ((1.0 - self.radius * self.radius / dist_light).sqrt() - 1.0);

        let phi = 2.0 * PI * rand_sq.x;
        let x = phi.cos() * (1.0 - z*z).sqrt();
        let y = phi.sin() * (1.0 - z*z).sqrt();

        let dir = x*u + y*v + z*w;

        Ray::new(
            h.p,
            dir.normalize() * dist_light,
            0,
        )
    }

    fn pdf(&self, r: &Ray) -> f64 {
        // check hit here for debug
        let cos_theta_max = (
            1.0 - self.radius*self.radius
                / (self.origin - r.origin).length_squared()
        ).sqrt();

        (2.0 * PI * (1.0 - cos_theta_max)).recip()
    }
}
