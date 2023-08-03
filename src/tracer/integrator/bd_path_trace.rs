use super::*;
use crate::tracer::material::Material;

/*
 * BUGS:
 * (1) MIS weight for polygon with transparent microfacet on low roughness delta
 * (2) Camera sampling raster coordinates
 */


/*
 * TODO:
 * (2) store directions in vertex?
 * (4) PBRT has no geometry term but we need it?
 * (9) need to modify vertex PDFs? maybe forget RR?
 * + this needs proper refactoring and cleanup...
 */
use vertex::Vertex;

/// Vertex abstraction
mod vertex;
/// Light and camera path generators
mod path_gen;
/// Multiple improtance sampling weights
mod mis;

pub fn integrate(scene: &Scene, camera: &Camera, r: Ray, x: i32, y: i32) -> Vec<FilmSample> {
    let light_path = path_gen::light_path(scene);
    let camera_path = path_gen::camera_path(scene, camera, r);

    let mut radiance = DVec3::ZERO;
    let mut samples = vec![];

    for s in 2..=light_path.len() {
        samples.push(connect_light_path(scene, camera, &camera_path, &light_path, s));
    }

    for t in 2..=camera_path.len() {
        for s in 0..=light_path.len() {
            radiance += connect_paths(
                scene,
                &light_path, s,
                &camera_path, t,
            );
        }
    }

    samples.push(FilmSample::new(radiance, x, y, false));
    samples
}

/// Paths starting from light and sample the camera (i.e. t == 1 and s > 1)
fn connect_light_path(
    scene: &Scene,
    camera: &Camera,
    camera_path: &[Vertex],
    light_path: &[Vertex],
    s: usize
) -> FilmSample {
    // assert!(s >= 2);

    let light_last = &light_path[s - 1];

    if light_last.is_delta() {
        return FilmSample::default();
    }

    let ro = camera.sample_towards(light_last.h.p, rand_utils::unit_square());
    let xo = ro.origin;
    let xi = light_last.h.p;
    let pdf = camera.sample_towards_pdf(&ro, xi);

    if pdf <= 0.0 {
        return FilmSample::default();
    }
    let v = -ro.dir;
    let vr = light_last.h.generate_ray(v);
    let t2 = xo.distance_squared(xi);

    if scene.hit(&vr).is_some_and(|h: Hit| h.t * h.t < t2 - crate::EPSILON) {
        return FilmSample::default();
    }

    let light_scnd_last = &light_path[s - 2];
    let mut sample = camera.importance_sample(&ro);
    sample.color /= pdf;
    let sampled_vertex = Some(Vertex::camera(ro.origin, sample.color));
    let camera_last = sampled_vertex.as_ref().unwrap();

    let shading_cosine = {
        let xn = light_scnd_last.h.p;
        let wi = (xn - xi).normalize();
        let ns = light_last.h.ns;
        let ng = light_last.h.ng;
        wi.dot(ng).abs() * light_last.shading_cosine(v, ns)
            / v.dot(ng).abs()
    };

    sample.color *= light_last.gathered
        * shading_cosine
        * light_last.bsdf(light_scnd_last, camera_last, Transport::Importance)
        * mis::mis_weight(light_path, s, camera_path, 1, sampled_vertex);

    sample
}

/// Connects a light subpath and a camera subpath.
/// Camera sampling not implemented i.e. camera paths of length 0 or 1 discarded.
/// Special logic if light path length 0 or 1.
fn connect_paths(
    scene: &Scene,
    light_path: &[Vertex],
    s: usize,
    camera_path: &[Vertex],
    t: usize,
) -> DVec3 {
    // assert!(t >= 2);

    // camera path ends on a light, but light path not empty
    if s != 0 && camera_path[t - 1].is_light() {
        return DVec3::ZERO;
    }

    let mut sampled_vertex: Option<Vertex> = None;

    let radiance = if s == 0 {
        // all vertices on camera path. check if last vertex is ON a light.
        let camera_last = &camera_path[t - 1];
        if !camera_last.is_light() {
            DVec3::ZERO
        } else {
            camera_last.gathered * camera_last.emittance()
        }
    } else if s == 1 {
        // just one vertex on the light path. instead of using it, we sample a
        // point on the same light.
        let camera_last = &camera_path[t - 1];
        // can't sample from delta and light as last exited early
        if camera_last.is_delta() {
            DVec3::ZERO
        } else {
            // .unwrap() not nice :(
            // just sample any random light?
            let light = light_path[0].h.light.unwrap();

            let xo = camera_last.h.p;
            let pdf_light = ObjectPdf::new(light, xo);

            match pdf_light.sample_direction(rand_utils::unit_square()) {
                None => DVec3::ZERO,
                Some(wi) => {
                    let ri = camera_last.h.generate_ray(wi);
                    match scene.hit_light(&ri, light) {
                        None => DVec3::ZERO,
                        Some(hi) => {
                            let ns = hi.ns;
                            let emittance = hi.material.emit(&hi)
                                / pdf_light.value_for(&ri, false);
                            sampled_vertex = Some(Vertex::light(
                                hi,
                                light,
                                emittance,
                                0.0,
                            ));
                            let light_last = sampled_vertex.as_ref().unwrap();
                            let bsdf = camera_last.bsdf(
                                &camera_path[t - 2],
                                light_last,
                                Transport::Radiance,
                            );

                            camera_last.gathered * bsdf * light_last.gathered
                                * camera_last.shading_cosine(wi, ns)
                        }
                    }
                }
            }
        }
    } else {
        // all other cases
        // assert!(s >= 2);
        // assert!(t >= 2);
        let light_last = &light_path[s - 1];
        let camera_last = &camera_path[t - 1];

        if camera_last.is_delta()
            || light_last.is_delta()
            || !visible(scene, &light_last.h, &camera_last.h) {
                DVec3::ZERO
            } else {
                let light_bsdf = light_last.bsdf(
                    &light_path[s - 2],
                    camera_last,
                    Transport::Importance,
                );
                let camera_bsdf = camera_last.bsdf(
                    &camera_path[t - 2],
                    light_last,
                    Transport::Radiance,
                );

                light_last.gathered * light_bsdf
                    * camera_bsdf * camera_last.gathered
                    * light_last.g(camera_last)
        }
    };

    let weight = if radiance.length_squared() == 0.0 {
        0.0
    } else {
        mis::mis_weight(light_path, s, camera_path, t, sampled_vertex)
    };

    radiance * weight
}

/// Is `h1` visible from `h2`?
fn visible(s: &Scene, h1: &Hit, h2: &Hit) -> bool {
    let xo = h1.p;
    let xi = h2.p;
    let r = h1.generate_ray(xi - xo);
    let wi = r.dir;

    if wi.dot(h1.ng) < crate::EPSILON {
        return false;
    }

    match s.hit(&r) {
        None => false,
        Some(h) => h.p.distance_squared(xi) < crate::EPSILON,
    }
}
