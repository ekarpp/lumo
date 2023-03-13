use super::*;

pub fn integrate(scene: &Scene, r: &Ray, depth: usize) -> DVec3 {
    /* put this here incase mirrors reflect from each other until infinity */
    if depth > PATH_TRACE_MAX_DEPTH {
        return DVec3::ZERO;
    }

    match scene.hit(r) {
        None => DVec3::new(0.0, 1.0, 0.0),
        Some(h) => {
            let material = h.object.material();
            match material.bsdf(&h, r) {
                None => material.emit(&h),
                /* mirror broken */
                Some(_) => {
                    material.emit(&h)
                        + material.albedo_at(h.p)
                        * light_at(scene, &h)
                }
            }
        }
    }
}

/// Randomly sample light from `h` and check if it is visible.
/// Currently only one shadow ray is shot.
fn light_at(scene: &Scene, h: &Hit) -> f64 {
    let light = scene.uniform_random_light();

    let pdf_light = ObjectPdf::new(
        light,
        h.p,
    );
    let pdf_scatter = CosPdf::new(
        h.norm,
    );
    let r = Ray::new(
        h.p,
        pdf_light.generate_dir(rand_utils::rand_unit_square()),
    );
    match scene.hit_light(&r, &light) {
        None => 0.0,
        Some(lh) => {
            pdf_scatter.pdf_val(r.dir, &lh)
                / pdf_light.pdf_val(r.dir, &lh)
        }
    }
}
