use super::*;

pub fn integrate(scene: &Scene, ro: Ray, x: i32, y: i32) -> FilmSample {
    let radiance = _integrate(scene, ro, 0);
    FilmSample::new(radiance, x, y, false)
}

const MAX_RECURSION: usize = 50;

fn _integrate(scene: &Scene, ro: Ray, depth: usize) -> DVec3 {
    match scene.hit(&ro) {
        None => DVec3::new(0.0, 0.0, 0.0),
        Some(ho) => {
            let material = ho.material;
            match material.bsdf_pdf(&ho, &ro) {
                None => material.emit(&ho),
                Some(scatter_pdf) => {
                    if material.is_specular() {
                        if depth > MAX_RECURSION {
                            return DVec3::ZERO;
                        }

                        match scatter_pdf.sample_direction(rand_utils::unit_square()) {
                            None => DVec3::ZERO,
                            Some(wi) => {
                                let ri = ho.generate_ray(wi);
                                let wi = ri.dir;
                                let wo = ro.dir;

                                let p_scatter = scatter_pdf.value_for(&ri, false);

                                if p_scatter <= 0.0 {
                                    // return something better?
                                    return DVec3::ZERO;
                                }

                                let ns = ho.ns;

                                let bsdf = material.bsdf_f(
                                    wo,
                                    wi,
                                    Transport::Radiance,
                                    &ho
                                );
                                let bsdf = if ho.is_medium() {
                                    // assume that mediums get sampled perfectly
                                    // according to the BSDF thus cancel out PDF
                                    bsdf * p_scatter
                                } else {
                                    bsdf
                                };

                                bsdf
                                    * scene.transmittance(&ho)
                                    * material.shading_cosine(wi, ns)
                                    * _integrate(scene, ri, depth + 1)
                                    / p_scatter
                            }
                        }
                    } else {
                        scene.transmittance(&ho)
                            * shadow_ray(
                                scene,
                                &ro,
                                &ho,
                                scatter_pdf.as_ref(),
                                rand_utils::unit_square()
                            )
                    }
                }
            }
        }
    }
}
