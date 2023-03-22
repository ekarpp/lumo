use super::*;

pub fn integrate(scene: &Scene, ro: &Ray) -> DVec3 {
    /* two mirrors next to each other might cause issues... */

    match scene.hit(ro) {
        None => DVec3::new(0.0, 1.0, 0.0),
        Some(ho) => {
            let material = ho.object.material();
            match material.bsdf_pdf(&ho, ro) {
                None => material.emit(&ho),
                Some(scatter_pdf) => {
                    match material {
                        Material::Diffuse(_) | Material::Microfacet(..) => {
                            JitteredSampler::new(SHADOW_SPLITS)
                                .fold(DVec3::ZERO, |acc, rand_sq| {
                                    acc + shadow_ray(scene,
                                                     ro,
                                                     &ho,
                                                     scatter_pdf.as_ref(),
                                                     rand_sq)
                                }) / SHADOW_SPLITS as f64

                        }
                        Material::Glass | Material::Mirror => {
                            let ri = scatter_pdf.sample_ray(
                                rand_utils::unit_square());
                            integrate(scene, &ri)
                                / scatter_pdf.value_for(&ri)
                        }
                        _ => DVec3::ZERO,
                    }
                }
            }
        }
    }
}
