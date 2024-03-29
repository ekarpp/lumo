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
            match material.bsdf_pdf(&ho, &ro) {
                None => material.emit(&ho),
                Some(scatter_pdf) => {
                    if material.is_specular() {
                        if depth > MAX_RECURSION {
                            return Color::BLACK;
                        }

                        match scatter_pdf.sample_direction(rand_utils::unit_square()) {
                            None => Color::BLACK,
                            Some(wi) => {
                                let ri = ho.generate_ray(wi);
                                let wi = ri.dir;
                                let wo = ro.dir;

                                let p_scatter = scatter_pdf.value_for(&ri, false);

                                if p_scatter <= 0.0 {
                                    // return something better?
                                    return Color::BLACK;
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
                                    * scene.transmittance(ho.t)
                                    * material.shading_cosine(wi, ns)
                                    * _integrate(scene, ri, depth + 1)
                                    / p_scatter
                            }
                        }
                    } else {
                        scene.transmittance(ho.t)
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
