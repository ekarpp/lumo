use super::*;

const RR_DEPTH: usize = 5;

pub fn integrate(
    scene: &Scene,
    mut ro: Ray,
    rng: &mut Xorshift,
    mut lambda: ColorWavelength,
    delta: Float,
    raster_xy: Vec2
) -> FilmSample {
    let mut last_specular = true;
    let mut radiance = Color::BLACK;
    let mut gathered = Color::WHITE;
    let mut depth = 0;

    while let Some(ho) = scene.hit(&ro, rng) {
        let material = ho.material;
        gathered *= scene.transmittance(&lambda, ho.t);
        let wo = -ro.dir;

        match material.bsdf_sample(wo, &ho, &mut lambda, rng.gen_float(), rng.gen_vec2()) {
            None => {
                if last_specular {
                    radiance += gathered * material.emit(&lambda, &ho)
                }
                break;
            }
            Some(wi) => {
                if !material.is_delta(&lambda) {
                    radiance += shadow_rays(
                        scene,
                        -ro.dir,
                        gathered,
                        &mut lambda,
                        &ho,
                        rng,
                    );
                }

                let ri = ho.generate_ray(wi);
                let wi = ri.dir;

                let p_scatter = material.bsdf_pdf(wo, wi, &ho, &mut lambda, false);
                // resample bad sample?
                if p_scatter <= 0.0 {
                    break;
                }

                let bsdf = material.bsdf_f(wo, wi, &lambda, Transport::Radiance, &ho);
                let bsdf = if ho.is_medium() {
                    // assume that mediums get sampled perfectly
                    // according to the BSDF and thus cancel out PDF
                    bsdf * p_scatter
                } else {
                    bsdf
                };

                let ns = ho.ns;
                gathered *= bsdf * material.shading_cosine(wi, ns)
                    / p_scatter;

                // russian roulette
                if depth >= RR_DEPTH {
                    let luminance = gathered.luminance(&lambda);
                    let rr_prob = (luminance / delta).min(1.0);
                    if rng.gen_float() > rr_prob {
                        break;
                    }
                    gathered /= rr_prob;
                }

                last_specular = material.is_specular();
                depth += 1;
                ro = ri;
            }
        }
    }

    FilmSample::new(radiance, lambda, raster_xy, false, depth)
}
