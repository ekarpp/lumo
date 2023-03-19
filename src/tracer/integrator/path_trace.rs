use super::*;

pub fn integrate(
             scene: &Scene,
             ro: &Ray,
             depth: usize,
             last_specular: bool
) -> DVec3 {
    // TODO: fix depth > 1
    if depth > 1 && rand_utils::rand_f64() < PATH_TRACE_RR {
        return DVec3::ZERO;
    }

    match scene.hit(ro) {
        None => DVec3::new(0.0, 0.0, 1.0),
        Some(ho) => {
            let material = ho.object.material();

            match material.bsdf_pdf(&ho, ro) {
                None => if last_specular {
                    material.emit(&ho)
                } else {
                    DVec3::ZERO
                },
                Some(scatter_pdf) => {
                    let no = ho.norm;
                    let ri = scatter_pdf.sample_ray(RandomShape::gen_2d(Square));
                    let wi = ri.dir;

                    // tmp, find better way to do this.
                    let tmp = matches!(material,
                                       Material::Glass | Material::Mirror);

                    let cos_theta = if tmp {
                        1.0
                    } else {
                        no.dot(wi.normalize()).abs()
                    };

                    shadow_ray(scene, ro, &ho, scatter_pdf.as_ref(),
                               RandomShape::gen_2d(Square))
                        + material.bsdf_f(ro, &ri, no)
                        * cos_theta
                        * integrate(scene, &ri, depth + 1, material.is_specular())
                        / (scatter_pdf.value_for(&ri)
                           * (1.0 - PATH_TRACE_RR))
                }
            }
        }
    }
}
