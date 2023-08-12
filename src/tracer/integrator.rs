use crate::{Transport, rand_utils};
use crate::tracer::camera::Camera;
use crate::tracer::film::FilmSample;
use crate::tracer::hit::Hit;
use crate::tracer::object::Sampleable;
use crate::tracer::pdfs::{ObjectPdf, Pdf};
use crate::tracer::ray::Ray;
use crate::tracer::scene::Scene;
use glam::{DVec2, DVec3};
use std::fmt;

mod bd_path_trace;
mod direct_light;
mod path_trace;

/// Enum to choose which integrator to use
pub enum Integrator {
    /// Implements the path tracing algorithm with
    /// Russian Roulette (With probability `p` terminate each path.
    /// Multiply contributions by reciprocal of `1-p`) and
    /// next event estimation (Importance sample light at each impact).
    PathTrace,
    /// Naive integrator that importance samples light once.
    DirectLight,
    /// Bidirectional path tracing.
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
    pub fn integrate(&self, s: &Scene, c: &Camera, x: i32, y: i32, r: Ray) -> Vec<FilmSample> {
        match self {
            Self::PathTrace => vec![path_trace::integrate(s, r, x, y)],
            Self::DirectLight => vec![direct_light::integrate(s, r, x, y)],
            Self::BDPathTrace => bd_path_trace::integrate(s, c, r, x, y),
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
    let material = ho.material;
    let xo = ho.p;
    let wo = ro.dir;
    let ns = ho.ns;

    let light = scene.uniform_random_light();

    let mut radiance = DVec3::ZERO;
    let pdf_light = ObjectPdf::new(light, xo);

    // refactor these to separate function?
    // sample light first
    radiance += match pdf_light.sample_direction(rand_sq) {
        None => DVec3::ZERO,
        Some(wi) => {
            let ri = ho.generate_ray(wi);
            match scene.hit_light(&ri, light) {
                None => DVec3::ZERO,
                Some(hi) => {
                    let p_light = pdf_light.value_for(&ri, false);
                    let p_scatter = pdf_scatter.value_for(&ri, false);
                    let wi = ri.dir;
                    // check bad samples?
                    let weight = p_light * p_light
                        / (p_light * p_light + p_scatter * p_scatter);

                    let bsdf = material.bsdf_f(wo, wi, Transport::Radiance, ho);
                    let bsdf = if ho.is_medium() {
                        // assume that mediums get sampled perfectly
                        // according to the BSDF and thus cancel out PDF
                        bsdf * p_scatter
                    } else {
                        bsdf
                    };

                    bsdf
                        * scene.transmittance(&hi)
                        * hi.material.emit(&hi)
                        * material.shading_cosine(wi, ns)
                        * weight
                        / p_light
                }
            }
        }
    };

    // then sample BSDF
    radiance += match pdf_scatter.sample_direction(rand_sq) {
        None => DVec3::ZERO,
        Some(wi) => {
            let ri = ho.generate_ray(wi);
            match scene.hit_light(&ri, light) {
                None => DVec3::ZERO,
                Some(hi) => {
                    let p_light = pdf_light.value_for(&ri, false);
                    let p_scatter = pdf_scatter.value_for(&ri, false);
                    let wi = ri.dir;
                    // check bad samples?
                    let weight = p_scatter * p_scatter
                        / (p_scatter * p_scatter + p_light * p_light);

                    let bsdf = material.bsdf_f(wo, wi, Transport::Radiance, ho);
                    let bsdf = if ho.is_medium() {
                        // assume that mediums get sampled perfectly
                        // according to the BSDF and thus cancel out PDF
                        bsdf * p_scatter
                    } else {
                        bsdf
                    };

                    bsdf
                        * scene.transmittance(&hi)
                        * hi.material.emit(&hi)
                        * material.shading_cosine(wi, ns)
                        * weight
                        / p_scatter
                }
            }
        }
    };

    radiance * scene.num_lights() as f64
}
