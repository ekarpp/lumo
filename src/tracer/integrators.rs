use crate::DVec3;
use crate::rand_utils;
use crate::consts::{SHADOW_RAYS, INTEGRATION_MAX_DEPTH};
use crate::pdfs::{Pdf, ObjectPdf, CosPdf};
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::scene::Scene;
use crate::tracer::object::Object;

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

    fn light_at(&self, h: &Hit) -> f64 {
        let light = &self.scene.objects[self.scene.lights[0]];

        let pdf_light = ObjectPdf::new(
            light,
            h.p,
            h.norm,
        );
        let pdf_scatter = CosPdf::new(
            h.norm,
        );

        let r = Ray::new(
            h.p,
            pdf_light.generate_dir(rand_utils::rand_unit_square()),
        );

        match self.scene.hit_light(&r, &light) {
            None => 0.0,
            Some(_lh) => {
                pdf_scatter.pdf_val(r.dir)
                    / pdf_light.pdf_val(r.dir)
            }
        }
/*
        self.scene.sample_lights_from(h).iter().map(|lh: &Hit| {
            let r = Ray::new(
                h.p,
                lh.p - h.p,
            );
            h.object.scatter_pdf(&r, h)
                / lh.object.pdf(&r, lh)
        }).fold(DVec3::ZERO, |acc, c| acc + c) / SHADOW_RAYS as f64
*/
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
                h.object.material().emit(&h)
                    + h.object.material().albedo_at(h.p) * self.light_at(&h)
            }

            /*
            {
                let material = h.object.material();
                material.emit(&h)
                    + material.albedo(&h)
                    * material.scatter(&h, &r).map_or(DVec3::ZERO, |sr: Ray| {
                        self.integrate(&sr, depth + 1)
                            * h.object.scatter_pdf(&sr, &h)
                            / h.object.scatter_pdf(&sr, &h)
                    })
            }
             */
        }
    }
}
