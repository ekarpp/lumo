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
         * where w is the direction from x to origin of this sphere.
         * w not normalized. */
        let w = (self.origin - h.p).normalize();
        let v = w.cross(h.norm).normalize();
        let u = w.cross(v);

        let dist_light = h.p.distance(self.origin);

        /* theta_max = solid angle */
        let sin_theta_max2 = self.radius * self.radius
            / (dist_light * dist_light);
        let cos_theta_max = (1.0 - sin_theta_max2).sqrt();

        let cos_theta = (1.0 - rand_sq.x) + rand_sq.x * cos_theta_max;
        let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();

        /* distance from hit point to sampled point on sphere */
        let dist_sp = dist_light * cos_theta
            - (self.radius*self.radius
               - dist_light*dist_light*sin_theta*sin_theta).max(0.0).sqrt();

        /* (alpha, phi) spherical coordinates of point in -uvw basis */
        let cos_alpha = (dist_light*dist_light + self.radius*self.radius
                         - dist_sp*dist_sp) / (2.0*dist_light*self.radius);
        let sin_alpha = (1.0 - cos_alpha*cos_alpha).max(0.0).sqrt();
        let phi = 2.0 * PI * rand_sq.y;

        let pt_sphere = DVec3::new(
            self.radius * cos_alpha * phi.sin(),
            self.radius * sin_alpha * phi.sin(),
            self.radius * phi.cos(),
        );

        /* starting from x, we get the direction towards
         * a random point on the hemisphere of this sphere
         * that is visible from x.
         * could do better and sample from the area of the hemisphere. */
        let dir = -DMat3::from_cols(u, v, w)
            * pt_sphere;

        Ray::new(
            h.p,
            dir.normalize() * dist_light,
            0,
        )
    }
}
