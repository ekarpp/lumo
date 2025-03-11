use super::*;
use crate::tracer::material::Material;

const RR_DEPTH: usize = 5;
const RR_MIN: Float = 0.05;

use vertex::Vertex;

/// Vertex abstraction
mod vertex;
/// Light and camera path generators
mod path_gen;
/// Multiple importance sampling weights
mod mis;
/// Helpers to convert between probability measures
mod measure;

#[cfg(test)]
mod mis_tests;

pub fn integrate(
    scene: &Scene,
    camera: &Camera,
    r: Ray,
    rng: &mut Xorshift,
    lambda: ColorWavelength,
    raster_xy: Vec2
) -> Vec<FilmSample> {
    let light_path = path_gen::light_path(scene, rng, &lambda);
    let camera_path = path_gen::camera_path(scene, camera, r, rng, &lambda);

    let mut radiance = Color::BLACK;
    let mut samples = vec![];

    // handle t == 1 cases
    for s in 2..=light_path.len() {
        match connect_light_path(scene, camera, rng, &lambda, &light_path[..s]) {
            None => (),
            Some(sample) => samples.push(sample),
        }
    }

    // handle s == 0 cases
    radiance += add_camera_path(scene, camera, &lambda, &camera_path);

    // handle s == 1 cases
    for t in 2..=camera_path.len() {
        radiance += connect_camera_path(scene, camera, rng, &lambda, &camera_path[..t])
    }

    // handle rest of the cases
    for t in 2..=camera_path.len() {
        for s in 2..=light_path.len() {
            radiance += connect_paths(
                scene, camera, rng, &lambda,
                &light_path[..s],
                &camera_path[..t],
            );
        }
    }

    samples.push( FilmSample::new(radiance, lambda, raster_xy, false) );
    samples
}

/// Paths starting from light and sample the camera (i.e. t == 1 and s > 1)
fn connect_light_path(
    scene: &Scene,
    camera: &Camera,
    rng: &mut Xorshift,
    lambda: &ColorWavelength,
    light_path: &[Vertex],
) -> Option<FilmSample> {
    let s = light_path.len();
    #[cfg(debug_assertions)]
    assert!(s >= 2);

    let light_last = &light_path[s - 1];
    // delta BxDFs don't work too well here
    if light_last.is_delta() {
        return None;
    }
    // sample direction
    let hi = &light_last.h;
    let xi = hi.p;
    let ri = camera.sample_towards(xi, rng.gen_vec2())?;
    let xo = ri.origin;
    let wi = ri.dir;
    // MIS checks this too
    let p_sct = light_last.bsdf_pdf(-wi, false);
    let p_imp = camera.pdf_importance(&ri, xi);
    if p_sct == 0.0 || p_imp == 0.0 {
        return None;
    }

    // visibility test
    let test = |h: Hit| (h.p - xi).abs().max_element() > crate::EPSILON.sqrt();
    if scene.hit(&ri, rng).is_none_or(test) {
        return None;
    }

    // get color
    match camera.sample_importance(&ri) {
        None => None,
        Some((mut color, raster_xy)) => {
            if color.is_black() {
                return None;
            }
            color /= p_imp;
            let p_xo = camera.pdf_xo(&ri);
            let camera_last = Vertex::camera(xo, p_xo, color / p_imp);
            let t2 = xo.distance_squared(xi);

            color *= light_last.gathered
                * scene.transmittance(lambda, t2.sqrt())
                * light_last.shading_cosine(-wi)
                * light_last.shading_correction(-wi)
                * light_last.f(&camera_last, lambda, Transport::Importance)
                * mis::weight(scene, camera, light_path, &[camera_last]);

            Some( FilmSample::new(color, lambda.clone(), raster_xy, true) )
        }
    }
}

fn add_camera_path(
    scene: &Scene,
    camera: &Camera,
    lambda: &ColorWavelength,
    camera_path: &[Vertex],
) -> Color {
    let t = camera_path.len();
    if !camera_path[t - 1].is_light() {
        Color::BLACK
    } else {
        let ct = &camera_path[t - 1];
        let rad = ct.gathered * ct.emittance(lambda);

        if rad.is_black() {
            Color::BLACK
        } else {
            rad * mis::weight(scene, camera, &[], camera_path)
        }
    }
}

fn connect_camera_path(
    scene: &Scene,
    camera: &Camera,
    rng: &mut Xorshift,
    lambda: &ColorWavelength,
    camera_path: &[Vertex],
) -> Color {
    let t = camera_path.len();
    if camera_path[t - 1].is_delta() || camera_path[t - 1].is_light() {
        return Color::BLACK;
    }

    let camera_last = &camera_path[t - 1];
    let light_idx = scene.sample_light(rng.gen_float());
    let (light, pdf_light) = scene.get_light(light_idx);

    let ho = &camera_last.h;
    let xo = ho.p;

    let wi = light.sample_towards(xo, rng.gen_vec2());
    // alternatively let MIS take care of this
    let p_sct = camera_last.bsdf_pdf(wi, false);
    if p_sct == 0.0 {
        return Color::BLACK;
    }

    let ri = ho.generate_ray(wi);
    match scene.hit_light(&ri, rng, light) {
        None => Color::BLACK,
        Some(hi) => {
            let xi = hi.p;
            let ngi = if !camera_last.is_surface() { wi } else { hi.ng };
            let p_lig = light.sample_towards_pdf(&ri, xi, ngi) * pdf_light;
            if p_lig == 0.0 {
                return Color::BLACK;
            }
            let wi = ri.dir;
            let pdf_origin = measure::sa_to_area(p_lig, xo, xi, wi, ngi);
            let emittance = hi.material.emit(lambda, &hi);
            let light_last = Vertex::light(
                hi,
                light_idx,
                emittance,
                pdf_origin,
            );
            let bsdf = camera_last.f(&light_last, lambda, Transport::Radiance);
            let tr = scene.transmittance(lambda, light_last.h.t);
            let cos_wi = camera_last.shading_cosine(wi);
            /* MB: medium bug. missing trace too */
            let radiance = camera_last.gathered * bsdf * emittance * tr * cos_wi
                / p_lig;

            radiance * mis::weight(scene, camera, &[light_last], camera_path)
        }
    }
}


/// Connects a light subpath and a camera subpath.
/// Special logic if light path length 0 or 1.
#[allow(clippy::too_many_arguments)]
fn connect_paths(
    scene: &Scene,
    camera: &Camera,
    rng: &mut Xorshift,
    lambda: &ColorWavelength,
    light_path: &[Vertex],
    camera_path: &[Vertex],
) -> Color {
    let s = light_path.len();
    let t = camera_path.len();

    #[cfg(debug_assertions)]
    {
        assert!(t >= 2);
        assert!(s >= 2);
    }

    let light_last = &light_path[s - 1];
    let camera_last = &camera_path[t - 1];

    if camera_last.is_delta()
        || camera_last.is_light()
        || light_last.is_delta()
        || !visible(scene, rng, &light_last.h, &camera_last.h) {
            return Color::BLACK;
        }
    let xc = camera_last.h.p;
    let xl = light_last.h.p;
    let wi = (xl - xc).normalize();
    // MIS checks these too
    let p_sct = camera_last.bsdf_pdf(wi, false)
        * light_last.bsdf_pdf(-wi, false);
    if p_sct == 0.0 {
        return Color::BLACK;
    }

    let light_bsdf = light_last.f(
        camera_last,
        lambda,
        Transport::Importance,
    );
    let camera_bsdf = camera_last.f(
        light_last,
        lambda,
        Transport::Radiance,
    );

    let radiance = light_last.gathered * light_bsdf * light_last.shading_cosine(-wi)
        * camera_last.gathered * camera_bsdf * camera_last.shading_cosine(wi)
        * scene.transmittance(lambda, xc.distance(xl))
        / xc.distance_squared(xl);

    if radiance.is_black() {
        Color::BLACK
    } else {
        radiance * mis::weight(scene, camera, light_path, camera_path)
    }
}

/// Is `h1` visible from `h2`?
fn visible(s: &Scene, rng: &mut Xorshift, h1: &Hit, h2: &Hit) -> bool {
    let xo = h1.p;
    let xi = h2.p;
    let ri = h1.generate_ray(xi - xo);
    let wi = ri.dir;

    if wi.dot(h1.ng) < crate::EPSILON {
        return false;
    }

    (xo.distance(xi) - s.hit_t(&ri, rng)).abs() < crate::EPSILON
}
