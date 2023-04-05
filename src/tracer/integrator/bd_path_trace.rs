use super::*;

pub fn integrate(scene: &Scene, ro: &Ray) -> DVec3 {
    let mut last_specular = true;
    let mut illuminance = DVec3::ZERO;
    let mut gathered = DVec3::ONE;
    let mut ray = Ray::new(ro.origin, ro.dir);

    while let Some(ho) = scene.hit(&ray) {
        let material = ho.object.material();

        match material.bsdf_pdf(&ho, &ray) {
            None => {
                if last_specular {
                    illuminance += gathered * material.emit(&ho)
                }
                break;
            }
            Some(scatter_pdf) => {
                let shadow = JitteredSampler::new(SHADOW_SPLITS)
                    .map(|rand_sq| shadow_ray(scene, &ray, &ho, scatter_pdf.as_ref(), rand_sq))
                    .sum::<DVec3>()
                    / SHADOW_SPLITS as f64;

                illuminance += gathered * shadow;

                if rand_utils::rand_f64() < PATH_TRACE_RR {
                    break;
                }

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

                        let cos_theta = ng.dot(wi).abs();

                        gathered *= material.bsdf_f(&ray, &ri, ns, ng)
                            * cos_theta
                            / (p_scatter * (1.0 - PATH_TRACE_RR));
                        last_specular = material.is_specular();
                        ray = ri;
                    }
                }
            }
        }
    }

    illuminance
}
