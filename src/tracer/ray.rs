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
                /* sphere normal*/
                let n = (p - h.sphere.origin).normalize();

                /* vector to light from hit point */
                let l = scene.light - p;
                /* l mirrored around sphere normal */
                let r = p - 2.0 * n * p.dot(n).max(0.0);

                let color = h.sphere.color;
                let spec_coeff = Vec3::splat(0.9);
                let ambient_coeff = Vec3::splat(0.75);
                let q = 0.25;
                let phong = color*ambient_coeff + (n.dot(l).max(0.0) * color
                    + r.dot(-p).max(0.0).powf(q) * spec_coeff)
                    / l.length_squared();

                let ray_to_light = Ray {
                    origin: p,
                    dir: l
                };


                if scene.hit_shadow(&ray_to_light) {
                }

                phong
            }
            None => {
                let u = self.dir.normalize();
                let t: f32 = 0.5*(u.y + 1.0);
                let c = Vec3::splat(1.0 - t) + t*Vec3::new(0.52, 0.81, 0.92);

                c / c.max_element()
            }
        }
    }
}
