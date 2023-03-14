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

            let xo = ho.p;
            let no = ho.norm;

            match material.bsdf(&ho, ro, RandomShape::gen_2d(Square)) {
                None => if last_specular {
                    material.emit(&ho)
                } else {
                    DVec3::ZERO
                },
                Some((ri, pdf_s)) => {
                    let wi = ri.dir;

                    let is_specular = matches!(
                        material,
                        Material::Mirror | Material::Glass
                    );

                    shadow_ray(scene, &ho, RandomShape::gen_2d(Square))
                        + material.brdf(xo)
                        * no.dot(wi.normalize()).abs()
                        * integrate(scene, &ri, depth + 1, is_specular)
                        / (pdf_s * (1.0 - PATH_TRACE_RR))
                }
            }
        }
    }
}
