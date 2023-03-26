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
                    if material.is_specular() {
                        let no = ho.norm;
                        let ri = scatter_pdf
                            .sample_ray(rand_utils::unit_square());
                        let wi = ri.dir;

                        // tmp, find better way to do this.
                        let tmp = matches!(material,
                                           Material::Glass | Material::Mirror);

                        let cos_theta = if tmp {
                            1.0
                        } else {
                            no.dot(wi).abs()
                        };

                        material.bsdf_f(ro, &ri, no)
                            * cos_theta
                            * integrate(scene, &ri)
                            / scatter_pdf.value_for(&ri)
                    } else {
                        JitteredSampler::new(SHADOW_SPLITS)
                            .fold(DVec3::ZERO, |acc, rand_sq| {
                                acc + shadow_ray(scene,
                                                 ro,
                                                 &ho,
                                                 scatter_pdf.as_ref(),
                                                 rand_sq)
                            }) / SHADOW_SPLITS as f64

                    }
                }
            }
        }
    }
}
