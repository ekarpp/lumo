use crate::rand_utils;
use crate::samplers::JitteredSampler;
use crate::tracer::hit::Hit;
use crate::tracer::pdfs::{ObjectPdf, Pdf};
use crate::tracer::ray::Ray;
use crate::tracer::scene::Scene;
use glam::{DVec2, DVec3};
use std::fmt;

mod bd_path_trace;
mod direct_light;
mod path_trace;

/// How many shadow rays per vertex in path tracer? Preferably square for
/// jittered sampler.
const SHADOW_SPLITS: u32 = 1;

/// Enum to choose which integrator to use
pub enum Integrator {
    /// Implements the path tracing algorithm with
    /// Russian Roulette (With probability `p` terminate each path.
    /// Multiply contributions by reciprocal of `1-p`) and
    /// next event estimation (Importance sample light at each impact).
    PathTrace,
    /// Naive integrator that importance samples light once.
    DirectLight,
    /// Bidirectional path tracing. Not implemented.
    BDPathTrace,
}

impl fmt::Display for Integrator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathTrace => write!(f, "path tracing"),
            Self::DirectLight => write!(f, "direct light integration"),
            Self::BDPathTrace => write!(f, "bidirectional path tracing"),
        }
    }
}

impl Integrator {
    /// Calls the corresponding integration function
    pub fn integrate(&self, s: &Scene, r: Ray) -> DVec3 {
        match self {
            Self::PathTrace => path_trace::integrate(s, r),
            Self::DirectLight => direct_light::integrate(s, r),
            Self::BDPathTrace => bd_path_trace::integrate(s, r),
        }
    }
}

/// Shoots a shadow ray towards random light from `ho`. MIS with `pdf_scatter`.
fn shadow_ray(
    scene: &Scene,
    ro: &Ray,
    ho: &Hit,
    pdf_scatter: &dyn Pdf,
    rand_sq: DVec2
) -> DVec3 {
    let material = ho.object.material();
    let xo = ho.p;
    let ng = ho.ng;
    let ns = ho.ns;

    let light = scene.uniform_random_light();

    let mut illuminance = DVec3::ZERO;
    let pdf_light = ObjectPdf::new(light, xo);

    // refactor these to separate function?
    // sample light first
    illuminance += match pdf_light.sample_ray(rand_sq) {
        None => DVec3::ZERO,
        Some(ri) => match scene.hit_light(&ri, light) {
            None => DVec3::ZERO,
            Some(_) => {
                let p_light = pdf_light.value_for(&ri);
                let p_scatter = pdf_scatter.value_for(&ri);
                let wi = ri.dir;

                let weight = p_light * p_light
                    / (p_light * p_light + p_scatter * p_scatter);

                // multiply by light emittance?
                material.bsdf_f(ro, &ri, ns, ng)
                    * ns.dot(wi).abs()
                    * weight
                    / (p_light + p_scatter)
            }
        }
    };

    // then sample BSDF
    illuminance += match pdf_scatter.sample_ray(rand_sq) {
        None => DVec3::ZERO,
        Some(ri) => match scene.hit_light(&ri, light) {
            None => DVec3::ZERO,
            Some(_) => {
                let p_light = pdf_light.value_for(&ri);
                let p_scatter = pdf_scatter.value_for(&ri);
                let wi = ri.dir;

                let weight = p_scatter * p_scatter
                    / (p_scatter * p_scatter + p_light * p_light);

                // multiply by light emittance?
                material.bsdf_f(ro, &ri, ns, ng)
                    * ns.dot(wi).abs()
                    * weight
                    / (p_scatter + p_light)
            }
        }
    };

    illuminance * scene.num_lights() as f64
}
