use super::*;

pub fn integrate(scene: &Scene, r: &Ray) -> DVec3 {
    /* two mirrors next to each other might cause issues... */

    match scene.hit(r) {
        None => DVec3::new(0.0, 1.0, 0.0),
        Some(h) => {
            let material = h.object.material();
            match material {
                Material::Diffuse(_) => {
                    shadow_ray(scene, &h, rand_utils::rand_unit_square())
                }
                Material::Glass | Material::Mirror => {
                    match material.bsdf(&h, r) {
                        Some(sr) => integrate(scene, &sr.ray),
                        None => DVec3::ZERO,
                    }
                }
                _ => material.emit(&h),

            }
        }
    }
}
