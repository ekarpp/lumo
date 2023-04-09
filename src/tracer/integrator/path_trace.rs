use super::*;

pub fn integrate(scene: &Scene, mut ro: Ray) -> DVec3 {
    let mut last_specular = true;
    let mut illuminance = DVec3::ZERO;
    let mut gathered = DVec3::ONE;
    let mut depth = 0;

    while let Some(ho) = scene.hit(&ro) {
        let material = ho.object.material();

        match material.bsdf_pdf(&ho, &ro) {
            None => {
                if last_specular {
                    illuminance += gathered * material.emit(&ho)
                }
                break;
            }
            Some(scatter_pdf) => {
                let shadow = if material.is_delta() {
                    DVec3::ZERO
                } else {
                    JitteredSampler::new(SHADOW_SPLITS)
                        .map(|rand_sq| {
                            shadow_ray(
                                scene,
                                &ro,
                                &ho,
                                scatter_pdf.as_ref(),
                                rand_sq
                            )
                        })
                        .sum::<DVec3>()
                        / SHADOW_SPLITS as f64
                };

                illuminance += gathered * shadow;

                match scatter_pdf.sample_ray(rand_utils::unit_square()) {
                    None => {
                        break;
                    }
                    Some(ri) => {
                        let wi = ri.dir;
                        let p_scatter = scatter_pdf.value_for(&ri);

                        // resample bad sample?
                        if p_scatter <= 0.0 {
                            break;
                        }

                        let ng = ho.ng;
                        let ns = ho.ns;

                        gathered *= material.bsdf_f(&ro, &ri, ns, ng)
                            * ns.dot(wi).abs()
                            / p_scatter;

                        // russian roulette
                        if depth > 3 {
                            let luminance = crate::rgb_to_luminance(gathered);
                            let rr_prob = (1.0 - luminance).max(0.05);
                            if rand_utils::rand_f64() < rr_prob {
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

    illuminance
}
