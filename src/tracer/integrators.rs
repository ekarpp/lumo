use crate::DVec3;
use crate::tracer::ray::Ray;
use crate::tracer::scene::Scene;

pub trait Integrator {
    fn integrate(&self, r: Ray) -> DVec3;
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
    fn integrate(&self, r: Ray) -> DVec3 {
        r.color(&self.scene)
    }
}
