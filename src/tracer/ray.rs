use glam::f64::DVec3;
use crate::tracer::scene::Scene;

pub struct Ray {
    pub origin: DVec3,
    pub dir: DVec3
}

impl Ray {
    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + t*self.dir
    }

    pub fn color(&self, scene: &Scene, depth: u32) -> DVec3 {
        if depth > 10 {
            return DVec3::ZERO;
        }

        match scene.hit(self) {
            Some(mut h) => {
                /* point where ray meets sphere */
                h.p = self.at(h.t);

                /* unit sphere normal */
                h.n = (h.p - h.sphere.origin).normalize();

                h.inside = h.n.dot(self.dir) > 0.0;
                if h.inside {
                    h.n = -h.n;
                }

                /* vector to light from hit point */
                h.l = scene.light - h.p;

                let ray_to_light = Ray {
                    origin: h.p,
                    dir: h.l
                };

                // ambient??
                let mut color = DVec3::ZERO;

                if !scene.hit_shadow(&ray_to_light) {
                    color += h.sphere.material.shade(&h);
                }

                match h.sphere.material.reflect(&h) {
                    Some(r) => color += r.color(scene, depth+1),
                    None => (),
                }
                match h.sphere.material.refract(&h, self) {
                    Some(r) => color += r.color(scene, depth+1),
                    None => (),
                }

                color
            }
            None => {
                /* add different scene types? night, day, etc.. */
                let u = self.dir.normalize();
                let t: f64 = 0.5*(u.y + 1.0);
                DVec3::splat(1.0 - t)*DVec3::ONE + t*DVec3::ZERO
            }
        }
    }
}
