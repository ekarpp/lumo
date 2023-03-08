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

    /* sample random ray in cone from h.p towards self */
    /* other objects might want random from sphere instead.. */
    pub fn sample_shadow(&self, h: &Hit, rand_disk: DVec2) -> Ray {
        let x = h.p;
        /* uvw-basis orthonormal basis,
         * where w is the direction from x to origin of this sphere.
         * w not normalized. */
        let w = self.origin - x;
        let v = w.cross(h.norm).normalize();
        let u = w.cross(v).normalize();

        let sin_theta_max2 = self.radius * self.radius / w.length_squared();
        let cos_theta_max = (1.0 - sin_theta_max2).sqrt();

        /* sample random point in unit disk. transform sampled
         *  point to the uvw-basis and add w.
         * (scaled by the cosine of the maximum cone angle).
         *
         * starting from x, we get the direction towards
         * a random point on the hemisphere of this sphere
         * that is visible from x.
         * could do better and sample from the area of the hemisphere. */
        let dir = DMat3::from_cols(u, v, w.normalize())
            * rand_disk.extend(0.0)
            + w * cos_theta_max;

        Ray::new(
            x,
            dir,
            0,
        )

        /*
        let sin_theta_max_sq = self.radius * self.radius / w.length_squared();
        let cos_theta_max = (1.0 - sin_theta_max_sq).sqrt();

        let r1 = rand_utils::rand_f64();
        let cos_theta = (1.0 - r1) + r1*cos_theta_max;
        let sin_theta = (1.0 - cos_theta).sqrt();
        let phi = rand_utils::rand_angle();
        */
    }
}

impl Object for Sphere {
    fn inside(&self, r: &Ray) -> bool {
        self.origin.distance_squared(r.origin + crate::EPSILON*r.dir)
            < self.radius*self.radius
    }

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
        if t < crate::EPSILON {
            t = (-half_b + disc_root) / a;
            if t < crate::EPSILON {
                return None;
            }
        }
        Hit::new(
            t,
            self,
            r,
        )
     }
}
