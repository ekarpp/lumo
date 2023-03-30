use super::*;

pub fn integrate(scene: &Scene, ro: &Ray, last_specularity: f64) -> DVec3 {
    match scene.hit(ro) {
        None => DVec3::ZERO,
        Some(ho) => {
            let material = ho.object.material();

            match material.bsdf_pdf(&ho, ro) {
                None => last_specularity * material.emit(&ho),
                Some(scatter_pdf) => {
                    // jittered sampler
                    let shadow = JitteredSampler::new(SHADOW_SPLITS)
                        .map(|rand_sq| shadow_ray(scene, ro, &ho, scatter_pdf.as_ref(), rand_sq))
                        .sum::<DVec3>()
                        / SHADOW_SPLITS as f64;

                    if rand_utils::rand_f64() < PATH_TRACE_RR {
                        return shadow;
                    }


                    match scatter_pdf.sample_ray(rand_utils::unit_square()) {
                        None => shadow,
                        Some(ri) => {
                            let wi = ri.dir;
                            let p_scatter = scatter_pdf.value_for(&ri);

                            // resample bad samples?
                            if p_scatter <= 0.0 {
                                return shadow;
                            }
                            let no = ho.norm;
                            // correct?
                            let cos_theta = if material.is_transparent() {
                                1.0
                            } else {
                                no.dot(wi).abs()
                            };

                            shadow
                                + material.bsdf_f(ro, &ri, no)
                                * cos_theta
                                * integrate(scene, &ri, material.specularity())
                                / (p_scatter * (1.0 - PATH_TRACE_RR))
                        }
                    }
                }
            }
        }
    }
}
