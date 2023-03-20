use super::*;

pub fn integrate(scene: &Scene, ro: &Ray, last_specular: bool) -> DVec3 {
    match scene.hit(ro) {
        None => panic!("ray escaped"),
        Some(ho) => {
            let material = ho.object.material();

            match material.bsdf_pdf(&ho, ro) {
                None => if last_specular {
                    material.emit(&ho)
                } else {
                    DVec3::ZERO
                },
                Some(scatter_pdf) => {
                    // jittered sampler
                    let shadow = JitteredSampler::new(SHADOW_SPLITS)
                        .fold(DVec3::ZERO, |acc, rand_sq| {
                            acc + shadow_ray(scene, ro, &ho, scatter_pdf.as_ref(),
                                             rand_sq)
                        }) / SHADOW_SPLITS as f64;

                    if rand_utils::rand_f64() < PATH_TRACE_RR {
                        return shadow;
                    }

                    let no = ho.norm;
                    let ri = scatter_pdf.sample_ray(rand_utils::unit_square());
                    let wi = ri.dir;

                    // tmp, find better way to do this.
                    let tmp = matches!(material,
                                       Material::Glass | Material::Mirror);

                    let cos_theta = if tmp {
                        1.0
                    } else {
                        no.dot(wi.normalize()).abs()
                    };

                    shadow + material.bsdf_f(ro, &ri, no)
                        * cos_theta
                        * integrate(scene, &ri, material.is_specular())
                        / (scatter_pdf.value_for(&ri)
                           * (1.0 - PATH_TRACE_RR))
                }
            }
        }
    }
}
