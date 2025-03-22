use super::*;

const MAX_RECURSION: usize = 50;

pub fn integrate(
    scene: &Scene,
    mut ro: Ray,
    rng: &mut Xorshift,
    mut lambda: ColorWavelength,
    raster_xy: Vec2
) -> FilmSample {
    let mut depth = 0;
    let mut radiance = Color::BLACK;
    let mut gathered = Color::WHITE;
    while let Some(ho) = scene.hit(&ro, rng) {
        let material = ho.material;
        gathered *= scene.transmittance(&lambda, ho.t);
        let wo = -ro.dir;

        match material.bsdf_sample(wo, &ho, &mut lambda, rng.gen_float(), rng.gen_vec2()) {
            None => {
                radiance += gathered * material.emit(&lambda, &ho);
                break;
            }
            Some(wi) => {
                if !material.is_specular() {
                    radiance += shadow_rays(
                        scene,
                        -ro.dir,
                        gathered,
                        &mut lambda,
                        &ho,
                        rng
                    );
                    break;
                }
                if depth >= MAX_RECURSION { break; }
                let ri = ho.generate_ray(wi);
                let wi = ri.dir;

                let p_scatter = material.bsdf_pdf(wo, wi, &ho, &lambda, false);
                if p_scatter <= 0.0 {
                    // return something better?
                    break;
                }

                let bsdf = material.bsdf_f(
                    wo,
                    wi,
                    &lambda,
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

                gathered *= bsdf * material.shading_cosine(wi, ns)
                    / p_scatter;
                depth += 1;
                ro = ri;
            }
        }
    }

    FilmSample::new(radiance, lambda, raster_xy, false, depth + 1)
}
