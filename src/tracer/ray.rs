use glam::f32::Vec3;
use crate::tracer::scene::Scene;

pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t*self.dir
    }

    pub fn color(&self, scene: &Scene) -> Vec3 {
        match scene.hit(self) {
            Some(h) => {
                /* point where ray meets sphere */
                let p = self.at(h.t);

                /* unit sphere normal */
                let n = (p - h.sphere.origin).normalize();

                /* squared distance to light from hit point */
                let l_len_sq = (scene.light - p).length_squared();

                /* unit vector to light from hit point */
                let l = (scene.light - p).normalize();

                let ray_to_light = Ray {
                    origin: p,
                    dir: l
                };

                let color = h.sphere.color;
                let spec_coeff = Vec3::splat(0.9);
                let q = 3.0;
                let phong = color*scene.ambient;

                if scene.hit_shadow(&ray_to_light) {
                    phong
                } else {
                    /* l mirrored around sphere normal */
                    let r = l - 2.0 * n * l.dot(n).max(0.0);

                    phong + (n.dot(l).max(0.0) * color
                             + r.dot(p).max(0.0).powf(q) * spec_coeff)
                        / l_len_sq
                }
            }
            None => {
                /* add different scene types? night, day, etc.. */
                let u = self.dir.normalize();
                let t: f32 = 0.5*(u.y + 1.0);
                Vec3::splat(1.0 - t)*Vec3::ONE + t*Vec3::ZERO
            }
        }
    }
}
