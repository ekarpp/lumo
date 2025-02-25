use super::*;

pub fn integrate(scene: &Scene, ro: Ray, raster_xy: Vec2) -> FilmSample {
    let radiance = _integrate(scene, ro, 0);
    FilmSample::new(radiance, raster_xy, false)
}

const MAX_RECURSION: usize = 50;

fn _integrate(scene: &Scene, ro: Ray, depth: usize) -> Color {
    match scene.hit(&ro) {
        None => Color::BLACK,
        Some(ho) => {
            let material = ho.material;
            let wo = -ro.dir;
            match material.bsdf_sample(wo, &ho, rand_utils::unit_square()) {
                None => material.emit(&ho),
                Some(wi) => {
                    if !material.is_specular() {
                        let radiance = shadow_ray(
                            scene,
                            -ro.dir,
                            &ho,
                            rand_utils::unit_square(),
                            rand_utils::unit_square(),
                        );

                        scene.transmittance(ho.t) * radiance
                    } else {
                        if depth > MAX_RECURSION {
                            return Color::BLACK;
                        }

                        let ri = ho.generate_ray(wi);
                        let wi = ri.dir;

                        let p_scatter = material.bsdf_pdf(wo, wi, &ho, false);
                        if p_scatter <= 0.0 {
                            // return something better?
                            return Color::BLACK;
                        }

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

                        let ns = ho.ns;

                        bsdf
                            * scene.transmittance(ho.t)
                            * material.shading_cosine(wi, ns)
                            * _integrate(scene, ri, depth + 1)
                            / p_scatter
                    }
                }
            }
        }
    }
}
