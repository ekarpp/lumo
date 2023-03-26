#![allow(dead_code)]
use crate::{DVec3, DVec2};
use std::fmt;
use crate::rand_utils;
use crate::samplers::JitteredSampler;
use crate::tracer::pdfs::{Pdf, ObjectPdf};
use crate::consts::{PATH_TRACE_RR, SHADOW_SPLITS};
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::ray::Ray;
use crate::scene::Scene;


mod path_trace;
mod direct_light;
mod bd_path_trace;

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
    pub fn integrate(&self, s: &Scene, r: &Ray) -> DVec3 {
        match self {
            Self::PathTrace => path_trace::integrate(s, r, true),
            Self::DirectLight => direct_light::integrate(s, r),
            Self::BDPathTrace => bd_path_trace::integrate(s, r),
        }
    }
}

/// Shoots a shadow ray towards random light from `ho`.
fn shadow_ray(
    scene: &Scene,
    ro: &Ray,
    ho: &Hit,
    _scatter_pdf: &dyn Pdf,
    rand_sq: DVec2
) -> DVec3 {
    let material = ho.object.material();

    if !material.is_diffuse() {
        DVec3::ZERO
    } else {
        let xo = ho.p;
        let no = ho.norm;
        let light = scene.uniform_random_light();

        let pdf_light = ObjectPdf::new(light, xo);
        let ri = pdf_light.sample_ray(rand_sq);
        let wi = ri.dir;

        match scene.hit_light(&ri, light) {
            None => DVec3::ZERO,
            Some(_) => {
                material.bsdf_f(ro, &ri, no)
                    * no.dot(wi).abs()
                    / pdf_light.value_for(&ri)
            }
        }
    }
}
