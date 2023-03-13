#![allow(dead_code)]
use crate::DVec3;
use crate::rand_utils;
use crate::consts::{PATH_TRACE_RR, PATH_TRACE_MAX_DEPTH};
use crate::pdfs::{Pdf, ObjectPdf, CosPdf};
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::ray::{Ray, ScatterRay};
use crate::tracer::scene::Scene;

pub trait Integrator {
    fn integrate(&self, r: &Ray, depth: usize) -> DVec3;
}

pub struct BiDirectionalPathTracingIntegrator {
    scene: Scene,
}

impl BiDirectionalPathTracingIntegrator {
    pub fn new(s: Scene) -> Self {
        Self {
            scene: s,
        }
    }
}

impl Integrator for BiDirectionalPathTracingIntegrator {
    fn integrate(&self, r: &Ray, depth: usize) -> DVec3 {
        DVec3::ZERO
    }
}

pub struct PathTracingIntegrator {
    scene: Scene,
}

impl Integrator for PathTracingIntegrator {
    fn integrate(&self, r: &Ray, depth: usize) -> DVec3 {
        self._integrate(r, depth, true)
    }

}

impl PathTracingIntegrator {
    pub fn new(s: Scene) -> Self {
        Self {
            scene: s,
        }
    }

    fn _integrate(&self, r: &Ray, depth: usize, last_specular: bool) -> DVec3 {
        if depth > 2 && rand_utils::rand_f64() < PATH_TRACE_RR {
            return DVec3::ZERO;
        }

        match self.scene.hit(r) {
            None => DVec3::new(0.0, 1.0, 0.0),
            Some(h) => {
                let material = h.object.material();

                match material.bsdf(&h, r) {
                    None => if last_specular {
                        material.emit(&h)
                    } else {
                        DVec3::ZERO
                    },
                    Some(sr) => {
                        let is_specular = match material {
                            Material::Mirror | Material::Glass => true,
                            _ => false,
                        };

                        self.shadow_ray(&h, &sr)
                            + material.albedo_at(h.p)
                            * self._integrate(&sr.ray, depth + 1, is_specular)
                            /* hit ok to pass here?? */
                            * sr.pdf.pdf_val(sr.ray.dir, &h)
                            / (1.0 - PATH_TRACE_RR)
                    }
                }
            }
        }
    }


    fn shadow_ray(&self, h: &Hit, sr: &ScatterRay) -> DVec3 {
        let material = h.object.material();
        match material {
            Material::Diffuse(_) => {
                let light = self.scene.uniform_random_light();

                let pdf_light = ObjectPdf::new(light, h.p);
                /* ray to sampled point on light */
                let r = Ray::new(
                    h.p,
                    pdf_light.generate_dir(
                        rand_utils::rand_unit_square()
                    ),
                );

                match self.scene.hit_light(&r, light) {
                    None => DVec3::ZERO,
                    Some(hl) => {
                        material.albedo_at(h.p)
                            * sr.pdf.pdf_val(r.dir, &hl)
                            / pdf_light.pdf_val(r.dir, &hl)
                    }
                }
            }
            _ => DVec3::ZERO,
        }
    }
}

pub struct DirectLightingIntegrator {
    scene: Scene,
}

impl Integrator for DirectLightingIntegrator {
    fn integrate(&self, r: &Ray, depth: usize) -> DVec3 {
        /* put this here incase mirrors reflect from each other until infinity */
        if depth > PATH_TRACE_MAX_DEPTH {
            return DVec3::ZERO;
        }

        match self.scene.hit(r) {
            None => DVec3::new(0.0, 1.0, 0.0),
            Some(h) => {
                let material = h.object.material();
                match material.bsdf(&h, r) {
                    None => material.emit(&h),
                    /* mirror broken */
                    Some(_) => {
                        material.emit(&h)
                            + material.albedo_at(h.p)
                            * self.light_at(&h)

                    }
                }
            }
        }
    }
}

impl DirectLightingIntegrator {
    pub fn new(s: Scene) -> Self {
        Self {
            scene: s,
        }
    }

    fn light_at(&self, h: &Hit) -> f64 {
        let light = self.scene.uniform_random_light();

        let pdf_light = ObjectPdf::new(
            light,
            h.p,
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
            Some(lh) => {
                pdf_scatter.pdf_val(r.dir, &lh)
                    / pdf_light.pdf_val(r.dir, &lh)
            }
        }
    }
}
