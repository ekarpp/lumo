use crate::DVec3;
use crate::consts::INTEGRATION_MAX_DEPTH;
use crate::tracer::ray::Ray;
use crate::tracer::scene::Scene;

pub trait Integrator {
    fn integrate(&self, r: &Ray, depth: usize) -> DVec3;
}

pub struct DirectLightingIntegrator {
    scene: Scene,
}

impl DirectLightingIntegrator {
    pub fn new(s: Scene) -> Self {
        Self {
            scene: s,
        }
    }
}

impl Integrator for DirectLightingIntegrator {
    fn integrate(&self, r: &Ray, depth: usize) -> DVec3 {
        if depth > INTEGRATION_MAX_DEPTH {
            return DVec3::ZERO;
        }

        match self.scene.hit(&r) {
            None => DVec3::new(0.0, 1.0, 0.0),
            Some(h) => {
                let material = h.object.material();
                material.emit(&h)
                    + material.albedo(&h)
                    * material.scatter(&h, &r).map_or(DVec3::ZERO, |sr: Ray| {
                        self.integrate(&sr, depth + 1)
                            * h.object.scatter_pdf(&sr, &h)
                            / h.object.scatter_pdf(&sr, &h)
                    })
            }
        }
    }
}
