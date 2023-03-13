#![allow(dead_code)]
use std::f64::consts::PI;
use crate::{DVec3, DVec2};
use crate::rand_utils;
use crate::consts::PATH_TRACE_RR;
use crate::pdfs::{Pdf, ObjectPdf};
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

/// Shoots a shadow ray towards random light from `h`. pass scatter pdf?
fn shadow_ray(scene: &Scene, h: &Hit, rand_sq: DVec2) -> DVec3 {
    let material = h.object.material();
    match material {
        Material::Diffuse(_) => {
            let light = scene.uniform_random_light();

            let pdf_light = ObjectPdf::new(light, h.p);
            /* ray to sampled point on light */
            let r = Ray::new(
                h.p,
                pdf_light.generate_dir(
                    rand_sq
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
