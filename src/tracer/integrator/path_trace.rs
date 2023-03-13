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

                    shadow_ray(scene, &h, &sr)
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

/// Shoots a shadow ray towards random light from `h`. `sr` has PDF?
fn shadow_ray(scene: &Scene, h: &Hit, sr: &ScatterRay) -> DVec3 {
    let material = h.object.material();
    match material {
        Material::Diffuse(_) => {
            let light = scene.uniform_random_light();

            let pdf_light = ObjectPdf::new(light, h.p);
            /* ray to sampled point on light */
            let r = Ray::new(
                h.p,
                pdf_light.generate_dir(
                    rand_utils::rand_unit_square()
                ),
            );

            match scene.hit_light(&r, light) {
                None => DVec3::ZERO,
                Some(hl) => {
                    material.albedo_at(h.p)
                        * sr.pdf.pdf_val(r.dir, &hl)
                        / pdf_light.pdf_val(r.dir, &hl)
                }
            }
        }
        _ => DVec3::ZERO,
    }
}
