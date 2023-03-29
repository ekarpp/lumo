use super::*;

pub fn integrate(scene: &Scene, ro: &Ray) -> DVec3 {
    /* two mirrors next to each other might cause issues... */

    match scene.hit(ro) {
        None => DVec3::new(0.0, 0.0, 0.0),
        Some(ho) => {
            let material = ho.object.material();
            match material.bsdf_pdf(&ho, ro) {
                None => material.emit(&ho),
                Some(scatter_pdf) => {
                    //  ¯\_(ツ)_/¯
                    if material.specularity() > 0.92 {
                        let no = ho.norm;
                        let ri = scatter_pdf.sample_ray(rand_utils::unit_square());
                        let wi = ri.dir;

                        let p_scatter = scatter_pdf.value_for(&ri);

                        if p_scatter <= 0.0 {
                            // return something better?
                            return DVec3::ZERO;
                        }

                        // correct?
                        let cos_theta = if material.is_transparent() {
                            1.0
                        } else {
                            no.dot(wi).abs()
                        };

                        material.bsdf_f(ro, &ri, no) * cos_theta * integrate(scene, &ri)
                            / scatter_pdf.value_for(&ri)
                    } else {
                        JitteredSampler::new(SHADOW_SPLITS).fold(DVec3::ZERO, |acc, rand_sq| {
                            acc + shadow_ray(scene, ro, &ho, scatter_pdf.as_ref(), rand_sq)
                        }) / SHADOW_SPLITS as f64
                    }
                }
            }
        }
    }
}
