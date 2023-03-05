use glam::f64::DVec3;
use crate::tracer::scene::Scene;

pub struct Ray {
    pub origin: DVec3,
    pub dir: DVec3
}

impl Ray {
    pub fn new(o: DVec3, d: DVec3) -> Ray {
        Ray {
            origin: o,
            dir: d,
        }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + t*self.dir
    }

    pub fn color(&self, scene: &Scene, depth: u32) -> DVec3 {
        if depth > 10 {
            return DVec3::ZERO;
        }

        match scene.hit(self) {
            Some(h) => {
                let mut color = DVec3::ZERO;

                /* better way than passing scene? */
                match h.object.material().shade(&h, scene) {
                    Some(c) => color += c,
                    None => (),
                }

                match h.object.material().reflect(&h) {
                    Some(r) => color += r.color(scene, depth+1),
                    None => (),
                }

                /* better way than passing self? */
                match h.object.material().refract(&h, self) {
                    Some(r) => color += r.color(scene, depth+1),
                    None => (),
                }

                color
            }
            None => {
                // for debug
                DVec3::new(0.0, 1.0, 0.0)
                /* add different scene types? night, day, etc.. */
                /*
                let u = self.dir.normalize();
                let t: f64 = 0.5*(u.y + 1.0);
                (1.0 - t)*DVec3::ONE + t*DVec3::ZERO
                */
            }
        }
    }
}
