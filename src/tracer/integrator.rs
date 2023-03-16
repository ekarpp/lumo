#![allow(dead_code)]
use crate::{DVec3, DVec2};
use crate::rand_utils;
use rand_utils::{RandomShape, RandomShape::Square};
use crate::pdfs::{Pdf, ObjectPdf};
use crate::consts::PATH_TRACE_RR;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::ray::Ray;
use crate::tracer::scene::Scene;

/// Implements the path tracing algorithm with
/// Russian Roulette (With probability `p` terminate each path.
/// Multiply contributions by reciprocal of `1-p`) and
/// next event estimation (Importance sample light at each impact).
mod path_trace;
/// Naive integrator that importance samples light once.
mod direct_light;
/// bidirectional path tracing
mod bd_path_trace;

/// Enum to choose which integrator to use
pub enum Integrator {
    PathTrace,
    DirectLight,
    BDPathTrace,
}

impl Integrator {
    pub fn integrate(&self, s: &Scene, r: &Ray) -> DVec3 {
        match self {
            Self::PathTrace => path_trace::integrate(s, r, 0, true),
            Self::DirectLight => direct_light::integrate(s, r),
            Self::BDPathTrace => bd_path_trace::integrate(s, r),
        }
    }
}

/// Shoots a shadow ray towards random light from `ho`.
fn shadow_ray(scene: &Scene, ho: &Hit, scatter_pdf: &dyn Pdf, rand_sq: DVec2)
              -> DVec3 {
    let material = ho.object.material();

    match material {
        Material::Diffuse(_) => {
            let xo = ho.p;
            let no = ho.norm;
            let light = scene.uniform_random_light();

            let pdf_light = ObjectPdf::new(light.as_ref(), xo);
            let ri = pdf_light.sample_ray(rand_sq);
            let wi = ri.dir;

            /* move this to object PDF */
            match scene.hit_light(&ri, light) {
                None => DVec3::ZERO,
                Some(hi) => {
                    material.bsdf_f(xo)
                        * no.dot(wi.normalize()).abs()
                        /* TODO: power heuristic, Veach & Guibas 95 */
                        * 0.5
                        / (pdf_light.value_for(wi, &hi)
                           + scatter_pdf.value_for(wi, &hi))
                }
            }
        }
        _ => DVec3::ZERO,
    }
}
