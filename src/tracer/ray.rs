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
                let p = self.at(h.t);
                let mut norm = (p - h.sphere.origin).normalize();

                let ray_to_light = Ray {
                    origin: p,
                    dir: scene.light - p
                };

                if scene.hit_shadow(&ray_to_light) {
                    norm *= 0.5;
                }

                norm
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
