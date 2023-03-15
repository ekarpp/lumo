#![allow(dead_code)]
use crate::{DVec3, DVec2};
use crate::rand_utils;
use rand_utils::{RandomShape, RandomShape::Square};
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
fn shadow_ray(scene: &Scene, ho: &Hit, rand_sq: DVec2) -> DVec3 {
    let material = ho.object.material();

    match material {
        Material::Diffuse(_) => {
            let xo = ho.p;
            let no = ho.norm;
            let light = scene.uniform_random_light();

            /* ray to sampled point on light and the corresponding
             * PDF value w.r.t solid angle */
            let (ri, pdf_w) = light.sample_towards(ho, rand_sq);
            let wi = ri.dir;

            match scene.hit_light(&ri, light) {
                None => DVec3::ZERO,
                Some(hi) => {
                    let xi = hi.p;
                    let ni = hi.norm;
                    /* PDF w.r.t to solid angle, need to change to area.
                     * HOW?
                     * dA = dw_i * cos(t_i) / dist_sq(xo, xi) */
                    let pdf_a = pdf_w * ni.dot(wi.normalize()).abs()
                        / xo.distance_squared(xi);

                    material.bsdf_f(xo)
                        * no.dot(wi.normalize()).abs()
                        / pdf_a
                }
            }
        }
        _ => DVec3::ZERO,
    }
}
