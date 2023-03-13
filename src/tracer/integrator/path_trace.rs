use super::*;

pub fn integrate(
             scene: &Scene,
             r: &Ray,
             depth: usize,
             last_specular: bool
) -> DVec3 {
    if depth > 2 && rand_utils::rand_f64() < PATH_TRACE_RR {
        return DVec3::ZERO;
    }

    match scene.hit(r) {
        None => DVec3::new(0.0, 1.0, 0.0),
        Some(h) => {
            let material = h.object.material();

            match material.bsdf(&h, r) {
                None => if last_specular {
                    material.emit(&h)
                } else {
                    DVec3::ZERO
                },
                Some(sr) => {
                    let is_specular = match material {
                        Material::Mirror | Material::Glass => true,
                        _ => false,
                    };

                    shadow_ray(scene, &h, rand_utils::rand_unit_square())
                        + material.albedo_at(h.p)
                        * integrate(scene, &sr.ray, depth + 1, is_specular)
                    /* hit ok to pass here?? */
                        * sr.pdf.pdf_val(sr.ray.dir, &h)
                        / (1.0 - PATH_TRACE_RR)
                }
            }
        }
    }
}
