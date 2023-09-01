use super::*;

pub fn integrate(scene: &Scene, mut ro: Ray, raster_xy: Vec2) -> FilmSample {
    let mut last_specular = true;
    let mut radiance = Color::BLACK;
    let mut gathered = Color::WHITE;
    let mut depth = 0;

    while let Some(ho) = scene.hit(&ro) {
        let material = ho.material;
        gathered *= scene.transmittance(ho.t);

        match material.bsdf_pdf(&ho, &ro) {
            None => {
                if last_specular {
                    radiance += gathered * material.emit(&ho)
                }
                break;
            }
            Some(scatter_pdf) => {
                if !material.is_delta() {
                    radiance += gathered
                        * shadow_ray(
                            scene,
                            &ro,
                            &ho,
                            scatter_pdf.as_ref(),
                            rand_utils::unit_square()
                        );
                }

                match scatter_pdf.sample_direction(rand_utils::unit_square()) {
                    None => break,
                    Some(wi) => {
                        let ri = ho.generate_ray(wi);
                        let wo = ro.dir;
                        let wi = ri.dir;
                        let p_scatter = scatter_pdf.value_for(&ri, false);

                        // resample bad sample?
                        if p_scatter <= 0.0 {
                            break;
                        }

                        let ns = ho.ns;

                        let bsdf = material.bsdf_f(wo, wi, Transport::Radiance, &ho);
                        let bsdf = if ho.is_medium() {
                            // assume that mediums get sampled perfectly
                            // according to the BSDF and thus cancel out PDF
                            bsdf * p_scatter
                        } else {
                            bsdf
                        };

                        gathered *= bsdf * material.shading_cosine(wi, ns)
                            / p_scatter;

                        // russian roulette
                        if depth > 3 {
                            let luminance = gathered.luminance();
                            let rr_prob = (1.0 - luminance).max(0.05);
                            if rand_utils::rand_float() < rr_prob {
                                break;
                            }
                            gathered /= 1.0 - rr_prob;
                        }

                        last_specular = material.is_specular();
                        depth += 1;
                        ro = ri;
                    }
                }
            }
        }
    }

    FilmSample::new(radiance, raster_xy, false)
}
