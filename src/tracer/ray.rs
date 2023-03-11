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
            Some(h) => {
                let material = h.object.material();
                material.emit(&h)
                    + material.albedo(&h)
                    * material.scatter(&h, self).map_or(DVec3::ZERO, |r: Ray| {
                        r.color(scene) * h.object.scatter_pdf(&r, &h)
                            / h.object.scatter_pdf(&r, &h)
                    })
            }
            /* bright green background for debug */
            None => DVec3::new(0.0, 1.0, 0.0),
        }
    }
}
