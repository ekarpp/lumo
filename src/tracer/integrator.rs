#![allow(dead_code)]
use std::f64::consts::PI;
use crate::DVec3;
use crate::rand_utils;
use crate::consts::PATH_TRACE_RR;
use crate::pdfs::{Pdf, ObjectPdf, CosPdf};
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::ray::{Ray, ScatterRay};
use crate::tracer::scene::Scene;

/// Implements the path tracing algorithm with
/// Russian Roulette and next event estimation.
mod path_trace;
/// Implements a direct light integrator.
mod direct_light;

/// Enum to choose which integrator to use
pub enum Integrator {
    PathTrace,
    DirectLight,
}

impl Integrator {
    pub fn integrate(&self, s: &Scene, r: &Ray) -> DVec3 {
        match self {
            Integrator::PathTrace => path_trace::integrate(s, r, 0, true),
            Integrator::DirectLight => direct_light::integrate(s, r),
        }
    }
}

/// Shoots a shadow ray towards random light from `h`.
fn shadow_ray(scene: &Scene, h: &Hit) -> DVec3 {
    let material = h.object.material();
    match material {
        Material::Diffuse(_) => {
            let light = scene.uniform_random_light();

            let pdf_light = ObjectPdf::new(light, h.p);
            /* ray to sampled point on light */
            let r = Ray::new(
                h.p,
                pdf_light.generate_dir(
                    rand_utils::rand_unit_square()
                ),
            );

            match scene.hit_light(&r, light) {
                None => DVec3::ZERO,
                Some(hl) => {
                    /* scatter pdf = cos / PI, might need to change in future
                     * if use different scattering */
                    material.albedo_at(h.p)
                        * h.norm.dot(r.dir.normalize())
                        * PI.recip()
                        / pdf_light.pdf_val(r.dir, &hl)
                }
            }
        }
        _ => DVec3::ZERO,
    }
}
