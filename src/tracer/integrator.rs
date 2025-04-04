use crate::{
    Transport, Vec2, Float,
    Normal, Point, Direction, Vec3, rng::Xorshift,
};
use crate::tracer::{
    camera::Camera, ColorWavelength, film::FilmSample, hit::Hit,
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
        rng: &mut Xorshift,
        delta: Float,
        raster_xy: Vec2,
    ) -> Vec<FilmSample> {
        #[cfg(debug_assertions)]
        assert!(delta > 0.0);

        let r = c.generate_ray(raster_xy, rng.gen_vec2());
        let lambda = ColorWavelength::sample(rng.gen_float());
        match self {
            Self::PathTrace => {
                vec![path_trace::integrate(s, r, rng, lambda, delta, raster_xy)]
            }
            Self::DirectLight => {
                vec![direct_light::integrate(s, r, rng, lambda, raster_xy)]
            }
            Self::BDPathTrace => {
                bd_path_trace::integrate(s, c, r, rng, lambda, delta, raster_xy)
            }
        }
    }
}

/// Shoots a shadow ray towards random light from `ho`. MIS with `pdf_scatter`.
#[inline]
fn shadow_rays(
    scene: &Scene,
    wo: Direction,
    gathered: Color,
    lambda: &mut ColorWavelength,
    ho: &Hit,
    rng: &mut Xorshift,
) -> Color {
    (0..scene.num_shadow_rays()).fold(Color::BLACK, |acc, _| {
        acc + gathered * single_shadow_ray(scene, wo, lambda, ho, rng)
    }) / scene.num_shadow_rays() as Float
}

fn single_shadow_ray(
    scene: &Scene,
    wo: Direction,
    lambda: &mut ColorWavelength,
    ho: &Hit,
    rng: &mut Xorshift,
) -> Color {
    let material = ho.material;
    let xo = ho.p;
    let (light, pdf_light) = scene.get_light(scene.sample_light(rng.gen_float()));

    let mut radiance = Color::BLACK;

    // sample light first
    radiance += {
        let wi = light.sample_towards(xo, rng.gen_vec2());
        let ri = ho.generate_ray(wi);
        match scene.hit_light(&ri, rng, light) {
            None => Color::BLACK,
            Some(hi) => {
                let xi = hi.p;
                let ng = hi.ng;
                let p_lig = light.sample_towards_pdf(&ri, xi, ng);
                let p_sct = material.bsdf_pdf(wo, wi, ho, lambda, false);
                mis_sample(scene, wo, wi, ho, hi, lambda, true, p_lig, p_sct)
            }
        }
    };

    // then sample BSDF
    let rand_u = rng.gen_float();
    let rand_sq = rng.gen_vec2();
    radiance += match material.bsdf_sample(wo, ho, lambda, rand_u, rand_sq) {
        None => Color::BLACK,
        Some(wi) => {
            let ri = ho.generate_ray(wi);
            match scene.hit_light(&ri, rng, light) {
                None => Color::BLACK,
                Some(hi) => {
                    let xi = hi.p;
                    let ng = hi.ng;
                    let p_lig = light.sample_towards_pdf(&ri, xi, ng);
                    let p_sct = material.bsdf_pdf(wo, wi, ho, lambda, false);
                    mis_sample(scene, wo, wi, ho, hi, lambda, false, p_lig, p_sct)
                }
            }
        }
    };

    radiance / pdf_light
}

fn mis_sample(
    scene: &Scene,
    wo: Direction,
    wi: Direction,
    ho: &Hit,
    hi: Hit,
    lambda: &ColorWavelength,
    li: bool,
    p_lig: Float,
    p_sct: Float
) -> Color {
    if p_lig == 0.0 || p_sct == 0.0 {
        return Color::BLACK;
    }

    let material = ho.material;
    let bsdf = material.bsdf_f(wo, wi, lambda, Transport::Radiance, ho);
    let bsdf = if ho.is_medium() {
        // assume that mediums get sampled perfectly
        // according to the BSDF and thus cancel out PDF
        bsdf * p_sct
    } else {
        bsdf
    };

    let ns = ho.ns;
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
}
