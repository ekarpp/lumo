use super::*;

pub fn integrate(scene: &Scene, ro: &Ray) -> DVec3 {
    /* two mirrors next to each other might cause issues... */

    match scene.hit(ro) {
        None => DVec3::new(0.0, 1.0, 0.0),
        Some(ho) => {
            let material = ho.object.material();
            match material {
                Material::Diffuse(_) => {
                    shadow_ray(scene, &ho, RandomShape::gen_2d(Square))
                }
                Material::Glass | Material::Mirror => {
                    match material.bsdf(&ho, ro, RandomShape::gen_2d(Square)) {
                        Some((ri, pdf_s)) => integrate(scene, &ri) / pdf_s,
                        None => DVec3::ZERO,
                    }
                }
                _ => material.emit(&ho),

            }
        }
    }
}
