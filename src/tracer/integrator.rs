use crate::{
    Transport, rand_utils, Vec2, Float,
    Normal, Point, Direction, Vec3
};
use crate::tracer::{
    camera::Camera, ColorWavelength, film::FilmSample, hit::Hit,
    object::Sampleable,
    ray::Ray, scene::Scene, Color
};
use std::fmt;

mod bd_path_trace;
mod direct_light;
mod path_trace;

/// Enum to choose which integrator to use
#[derive(Clone)]
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

impl Default for Integrator {
    fn default() -> Self { Self::PathTrace }
}

impl Integrator {
    /// Calls the corresponding integration function
    pub fn integrate(
        &self,
        s: &Scene,
        c: &Camera,
        raster_xy: Vec2,
    ) -> Vec<FilmSample> {
        let r = c.generate_ray(raster_xy);
        let lambda = ColorWavelength::sample(rand_utils::rand_float());
        match self {
            Self::PathTrace => vec![path_trace::integrate(s, r, lambda, raster_xy)],
            Self::DirectLight => vec![direct_light::integrate(s, r, lambda, raster_xy)],
            Self::BDPathTrace => bd_path_trace::integrate(s, c, r, lambda, raster_xy),
        }
    }
}

/// Shoots a shadow ray towards random light from `ho`. MIS with `pdf_scatter`.
fn shadow_ray(
    scene: &Scene,
    wo: Direction,
    lambda: &ColorWavelength,
    ho: &Hit,
    rand_sq0: Vec2,
    rand_sq1: Vec2,
) -> Color {
    let material = ho.material;
    let xo = ho.p;
    let ns = ho.ns;

    let light = scene.uniform_random_light();

    let mut radiance = Color::BLACK;

    let mis_sample = |hi: Hit, wi: Direction, li: bool, p_lig: Float, p_sct: Float| -> Color {
        if p_lig == 0.0 || p_sct == 0.0 {
            return Color::BLACK;
        }

        let bsdf = material.bsdf_f(wo, wi, lambda, Transport::Radiance, ho);
        let bsdf = if ho.is_medium() {
            // assume that mediums get sampled perfectly
            // according to the BSDF and thus cancel out PDF
            bsdf * p_sct
        } else {
            bsdf
        };

        let heuristic = |p: Float| -> Float { p * p };
        let denom = heuristic(p_lig) + heuristic(p_sct);
        let weight = if li {
            heuristic(p_lig) / denom
        } else {
            heuristic(p_sct) / denom
        };
        let p_denom = if li {
            p_lig
        } else {
            p_sct
        };

        bsdf
            * scene.transmittance(lambda, hi.t)
            * hi.material.emit(lambda, &hi)
            * material.shading_cosine(wi, ns)
            * weight
            / p_denom
    };

    // sample light first
    radiance += {
        let wi = light.sample_towards(xo, rand_sq0);
        let ri = ho.generate_ray(wi);
        match scene.hit_light(&ri, light) {
            None => Color::BLACK,
            Some(hi) => {
                let xi = hi.p;
                let ng = hi.ng;
                let p_lig = light.sample_towards_pdf(&ri, xi, ng);
                let p_sct = material.bsdf_pdf(wo, wi, ho, false);
                mis_sample(hi, wi, true, p_lig, p_sct)
            }
        }
    };

    // then sample BSDF
    radiance += match material.bsdf_sample(wo, ho, rand_sq1) {
        None => Color::BLACK,
        Some(wi) => {
            let ri = ho.generate_ray(wi);
            match scene.hit_light(&ri, light) {
                None => Color::BLACK,
                Some(hi) => {
                    let xi = hi.p;
                    let ng = hi.ng;
                    let p_lig = light.sample_towards_pdf(&ri, xi, ng);
                    let p_sct = material.bsdf_pdf(wo, wi, ho, false);
                    mis_sample(hi, wi, false, p_lig, p_sct)
                }
            }
        }
    };

    radiance * scene.num_lights() as Float
}
