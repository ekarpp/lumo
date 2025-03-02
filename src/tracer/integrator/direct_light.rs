use super::*;

pub fn integrate(
    scene: &Scene,
    ro: Ray,
    rng: &mut Xorshift,
    lambda: ColorWavelength,
    raster_xy: Vec2
) -> FilmSample {
    let radiance = _integrate(scene, ro, rng, &lambda, 0);
    FilmSample::new(radiance, lambda, raster_xy, false)
}

const MAX_RECURSION: usize = 50;

fn _integrate(
    scene: &Scene,
    ro: Ray,
    rng: &mut Xorshift,
    lambda: &ColorWavelength,
    depth: usize
) -> Color {
    match scene.hit(&ro, rng) {
        None => Color::BLACK,
        Some(ho) => {
            let material = ho.material;
            let wo = -ro.dir;

            match material.bsdf_sample(wo, &ho, rng.gen_float(), rng.gen_vec2()) {
                None => material.emit(lambda, &ho),
                Some(wi) => {
                    if !material.is_specular() {
                        let radiance = shadow_ray(
                            scene,
                            -ro.dir,
                            lambda,
                            &ho,
                            rng,
                        );

                        scene.transmittance(lambda, ho.t) * radiance
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
                            lambda,
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
                            * scene.transmittance(lambda, ho.t)
                            * material.shading_cosine(wi, ns)
                            * _integrate(scene, ri, rng, lambda, depth + 1)
                            / p_scatter
                    }
                }
            }
        }
    }
}
