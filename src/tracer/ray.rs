use crate::DVec3;
use crate::consts::RAY_MAX_DEPTH;
use crate::tracer::scene::Scene;

pub struct Ray {
    pub origin: DVec3,
    /* should not be normalized. go through code to verify */
    pub dir: DVec3,
    pub depth: usize,
}

impl Ray {
    pub fn new(origin: DVec3, dir: DVec3, depth: usize) -> Self {
        Self {
            origin: origin,
            dir: dir,
            depth: depth + 1,
        }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + t*self.dir
    }

    pub fn color(&self, scene: &Scene) -> DVec3 {
        if self.depth > RAY_MAX_DEPTH {
            return DVec3::ZERO;
        }

        match scene.hit(self) {
            Some(h) => h.object.material().color(&h, scene, self),
            // bright green for debug
            None => DVec3::new(0.0, 1.0, 0.0),
            /*{
                /* add different scene types? night, day, etc.. */
                let u = self.dir.normalize();
                let t: f64 = 0.5*(u.y + 1.0);
                (1.0 - t)*DVec3::ONE + t*DVec3::ZERO
            }*/
        }
    }
}
