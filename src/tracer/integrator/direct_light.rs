use super::*;

pub fn integrate(scene: &Scene, ro: Ray) -> DVec3 {
    /* two mirrors next to each other might cause issues... */

    match scene.hit(&ro) {
        None => DVec3::new(0.0, 0.0, 0.0),
        Some(ho) => {
            let material = ho.object.material();
            match material.bsdf_pdf(&ho, &ro) {
                None => material.emit(&ho),
                Some(scatter_pdf) => {
                    if material.is_specular() {
                        match scatter_pdf.sample_direction(rand_utils::unit_square()) {
                            None => DVec3::ZERO,
                            Some(wi) => {
                                let ri = ho.generate_ray(wi);
                                let wi = ri.dir;

                                let p_scatter = scatter_pdf.value_for(&ri);

                                if p_scatter <= 0.0 {
                                    // return something better?
                                    return DVec3::ZERO;
                                }

                                let ng = ho.ng;
                                let ns = ho.ns;

                                material.bsdf_f(&ro, &ri, ns, ng)
                                    * ns.dot(wi).abs()
                                    * integrate(scene, ri)
                                    / p_scatter
                            }
                        }
                    } else {
                        JitteredSampler::new(SHADOW_SPLITS).fold(DVec3::ZERO, |acc, rand_sq| {
                            acc + shadow_ray(scene, &ro, &ho, scatter_pdf.as_ref(), rand_sq)
                        }) / SHADOW_SPLITS as f64
                    }
                }
            }
        }
    }
}
