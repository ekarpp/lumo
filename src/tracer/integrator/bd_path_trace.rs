use super::*;
use crate::tracer::material::Material;

/*
 * TODO:
 * (2) store directions in vertex?
 * (4) PBRT has no geometry term but we need it?
 * (9) need to modify vertex PDFs? maybe forget RR?
 * + this needs proper refactoring and cleanup...
 */

/// Abstraction of a vertex in the paths
struct Vertex<'a> {
    h: Hit<'a>,
    gathered: DVec3,
    pdf_fwd: f64,
    pdf_bck: f64,
}

impl<'a> Vertex<'a> {
    /// Camera vertex
    pub fn camera(xo: DVec3, gathered: DVec3) -> Self {
        let h = Hit::new(
            0.0,
            &Material::Blank,
            DVec3::NEG_X,
            xo,
            DVec3::ZERO,
            DVec3::X,
            DVec3::X,
            DVec2::X,
        ).unwrap();
        Self {
            h,
            gathered,
            pdf_bck: 0.0,
            pdf_fwd: 1.0,
        }
    }

    /// Light vertex
    pub fn light(mut h: Hit<'a>, light: &'a dyn Sampleable, gathered: DVec3, pdf_fwd: f64) -> Self {
        h.light = Some(light);
        Self {
            h,
            gathered,
            pdf_fwd,
            pdf_bck: 0.0,
        }
    }

    /// Surface vertex
    pub fn surface(
        h: Hit<'a>,
        gathered: DVec3,
    ) -> Self {
        Self {
            h,
            gathered,
            pdf_fwd: 0.0,
            pdf_bck: 0.0,
        }
    }

    /// Are we on a surface with delta material?
    pub fn is_delta(&self) -> bool {
        self.h.material.is_delta()
    }

    /// Computes BSDF at hit of `self`
    pub fn bsdf(&self, prev: &Vertex, next: &Vertex) -> DVec3 {
        // TODO (2)
        let wo = (self.h.p - prev.h.p).normalize();
        let wi = (next.h.p - self.h.p).normalize();

        self.h.material.bsdf_f(wo, wi, Transport::Radiance, &self.h)
    }

    /// Converts solid angle `pdf` to area PDF
    fn solid_angle_to_area(&self, pdf: f64, xi: DVec3, ng: DVec3) -> f64 {
        let wi = (xi - self.h.p).normalize();
        pdf * wi.dot(ng).abs() / xi.distance_squared(self.h.p)
    }
}

pub fn integrate(scene: &Scene, camera: &Camera, r: Ray, x: i32, y: i32) -> Vec<FilmSample> {
    let light_path = light_path(scene);
    let camera_path = camera_path(scene, r);
    let mut sample = FilmSample::new(DVec3::ZERO, x, y);
    let mut samples = vec![];

    for s in 2..=light_path.len() {
        samples.push(connect_light_path(scene, camera, &camera_path, &light_path, s));
    }

    for t in 2..=camera_path.len() {
        for s in 0..=light_path.len() {
            sample.color += connect_paths(
                scene,
                &light_path, s,
                &camera_path, t,
            );
        }
    }

    samples.push(sample);
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
    let xi = light_last.h.p;
    let pdf = camera.sample_towards_pdf(&ro, xi);

    if pdf <= 0.0 {
        return FilmSample::default();
    }
    let wi = -ro.dir;
    let v = light_last.h.generate_ray(wi);
    let t2 = v.origin.distance_squared(ro.origin);

    if scene.hit(&v).is_some_and(|h: Hit| h.t * h.t < t2 - crate::EPSILON) {
        return FilmSample::default();
    }

    let light_scnd_last = &light_path[s - 2];
    let mut sample = camera.importance_sample(&ro);
    sample.color /= pdf;
    let sampled_vertex = Some(Vertex::camera(ro.origin, sample.color));
    let camera_last = sampled_vertex.as_ref().unwrap();
    let ns = light_last.h.ns;

    sample.color *= light_last.gathered
        * light_last.h.material.shading_cosine(wi, ns)
        * light_last.bsdf(light_scnd_last, camera_last)
        * mis_weight(light_path, s, camera_path, 1, sampled_vertex);

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

    let mut sampled_vertex: Option<Vertex> = None;

    let radiance = if s == 0 {
        // all vertices on camera path. check if last vertex is ON a light.
        let camera_last = &camera_path[t - 1];
        if !camera_last.h.is_light() {
            DVec3::ZERO
        } else {
            let emittance = camera_last.h.material.emit(&camera_last.h);
            camera_last.gathered * emittance
        }
    } else if s == 1 {
        // just one vertex on the light path. instead of using it, we sample a
        // point on the same (TODO: at the moment uniform random) light.
        let camera_last = &camera_path[t - 1];
        if camera_last.h.is_light() || camera_last.h.material.is_delta() {
            DVec3::ZERO
        } else {
            // .unwrap() not nice :(
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
                            let emittance = hi.material.emit(&hi);
                            sampled_vertex = Some(Vertex::light(
                                hi,
                                light,
                                emittance,
                                0.0,
                            ));
                            let light_last = sampled_vertex.as_ref().unwrap();
                            let bsdf = camera_last.bsdf(
                                &camera_path[t - 2],
                                light_last
                            );

                            // TODO (4)
                            // geometry term not used in PBRT, but it breaks w/o
                            camera_last.gathered * bsdf * emittance
                                * camera_last.h.material.shading_cosine(wi, ns)
                                * geometry_term(light_last, camera_last)
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

        if camera_last.h.is_light() || !unoccluded(scene, &light_last.h, &camera_last.h) {
            DVec3::ZERO
        } else {
            let light_bsdf = light_last.bsdf(&light_path[s - 2], camera_last);
            let camera_bsdf = camera_last.bsdf(&camera_path[t - 2], light_last);

            light_last.gathered * light_bsdf
                * camera_bsdf * camera_last.gathered
                * geometry_term(light_last, camera_last)
        }
    };

    let weight = if radiance.length_squared() == 0.0 {
        0.0
    } else {
        mis_weight(light_path, s, camera_path, t, sampled_vertex)
    };

    radiance * weight
}

/// Are there any objects blocking from `p1` to `p2`
fn unoccluded(s: &Scene, h1: &Hit, h2: &Hit) -> bool {
    let r = h1.generate_ray(h2.p - h1.p);
    let h = s.hit(&r);

    match h {
        None => false,
        Some(h) => h.p.distance_squared(h2.p) < crate::EPSILON,
    }
}

/// Area pdf at `curr` ???
fn pdf_area(prev: &Vertex, curr: &Vertex, next: &Vertex) -> f64 {
    let ho = &curr.h;

    if ho.material.is_delta() {
        return 0.0;
    }

    let xo = prev.h.p;
    let xi = curr.h.p;
    let wo = xi - xo;
    let ro = Ray::new(xo, wo);
    let wi = next.h.p - xi;
    let ri = Ray::new(xi, wi);
    let sa = match ho.material.bsdf_pdf(ho, &ro) {
        None => 0.0,
        Some(pdf) => pdf.value_for(&ri, false),
    };

    sa * ri.dir.dot(next.h.ng).abs() / wi.length_squared()
}

/// Computes the MIS weight for the chosen sample strategy. PBRT, what orig paper
fn mis_weight(
    light_path: &[Vertex],
    s: usize,
    camera_path: &[Vertex],
    t: usize,
    sampled_vertex: Option<Vertex>,
) -> f64 {
    // assert!(t != 0)
    // if `sampled_vertex.is_some()` then t == 1 XOR s == 1

    if s + t == 2 {
        return 1.0;
    }

    let map0 = |pdf: f64| {
        if pdf == 0.0 { 1.0 } else { pdf }
    };

    let mut sum_ri = 0.0;
    let mut ri = 1.0;

    // applies the updated PDF for camera_last of the connection
    if t > 0 {
        let ct = &camera_path[t - 1];
        let pdf_prev = if s == 0 {
            // probability for the origin. uniformly sampled on light surface
            ct.h.light.map_or(0.0, |light| 1.0 / light.area())
        } else if s == 1 {
            let ls = sampled_vertex.as_ref().unwrap_or(&light_path[s - 1]);
            match ls.h.light {
                None => 0.0,
                Some(light) => {
                    let xo = ls.h.p;
                    let xi = ct.h.p;
                    let wi = xi - xo;
                    let ri = Ray::new(xo, wi);
                    // normalized
                    let wi = ri.dir;
                    let ng = ls.h.ng;
                    let (_, pdf_dir) = light.sample_leaving_pdf(&ri, ng);
                    let ng = ct.h.ng;
                    // convert solid angle to area
                    pdf_dir * wi.dot(ng).abs() / xo.distance_squared(xi)
                }
            }
        } else {
            let ls = &light_path[s - 1];
            let ls_m = &light_path[s - 2];
            pdf_area(ls_m, ls, ct)
        };

        ri *= map0(pdf_prev) / map0(ct.pdf_fwd);
        sum_ri += ri;
    }

    // applies the updated PDF for camera t - 2 using the connection
    if t > 1 {
        let ct = &camera_path[t - 1];
        let ct_m = &camera_path[t - 2];
        let pdf_prev = if s == 0 {
            match ct.h.light {
                None => 0.0,
                Some(light) => {
                    let xo = ct.h.p;
                    let xi = ct_m.h.p;
                    let wi = xi - xo;
                    let ri = Ray::new(xo, wi);
                    // normalized
                    let wi = ri.dir;
                    let ng = ct.h.ng;
                    let (_, pdf_dir) = light.sample_leaving_pdf(&ri, ng);
                    let ng = ct_m.h.ng;
                    // convert solid angle to area
                    pdf_dir * wi.dot(ng).abs() / xo.distance_squared(xi)
                }
            }
        } else {
            let ls = sampled_vertex.as_ref().unwrap_or(&light_path[s - 1]);
            pdf_area(ls, ct, ct_m)
        };
        ri *= map0(pdf_prev) / map0(ct_m.pdf_fwd);
        sum_ri += ri;
    }

    // vertices in camera path
    for i in (1..t.max(2) - 2).rev() {
        ri *= map0(camera_path[i].pdf_bck) / map0(camera_path[i].pdf_fwd);
        sum_ri += ri;
    }

    let mut ri = 1.0;

    // applies the updated PDF at light_last using the connection
    if s > 0 {
        let ls = &light_path[s - 1];
        let pdf_prev = if t < 2 {
            // probability that the direction got sampled from camera.
            // should be 1.0? yes.
            1.0
        } else {
            let ct = &camera_path[t - 1];
            let ct_m = &camera_path[t - 2];
            pdf_area(ct_m, ct, ls)
        };
        ri *= map0(pdf_prev) / map0(ls.pdf_fwd);
        sum_ri += ri;
    }

    // applies the updated PDF at light_last using the connection
    if s > 1 {
        let ls = &light_path[s - 1];
        let ls_m = &light_path[s - 2];
        let ct = sampled_vertex.as_ref().unwrap_or(&camera_path[t - 1]);

        let pdf_prev = pdf_area(ct, ls, ls_m);

        ri *= map0(pdf_prev) / map0(ls_m.pdf_fwd);
        sum_ri += ri;
    }

    // vertices in light path
    for i in (1..s.max(2) - 2).rev() {
        ri *= map0(light_path[i].pdf_bck) / map0(light_path[i].pdf_fwd);
        sum_ri += ri;
    }

    1.0 / (1.0 + sum_ri)
}

/// Geometry term ???
fn geometry_term(v1: &Vertex, v2: &Vertex) -> f64 {
    let v1_xo = v1.h.p;
    let v2_xo = v2.h.p;
    let v1_ns = v1.h.ns;
    let v2_ns = v2.h.ns;

    let wi = (v1_xo - v2_xo).normalize();
    let wi_length_squared = v1_xo.distance_squared(v2_xo);

    v1_ns.dot(wi).abs() * v2_ns.dot(wi).abs() / wi_length_squared
}

/// Generates a ray path starting from the camera
fn camera_path(scene: &Scene, r: Ray) -> Vec<Vertex> {
    let gathered = DVec3::ONE;
    let root = Vertex::camera(r.origin, gathered);

    walk(scene, r, root, gathered, 1.0, Transport::Radiance)
}

/// Generates a ray path strating from a light
fn light_path(scene: &Scene) -> Vec<Vertex> {
    let light = scene.uniform_random_light();
    let pdf_light = 1.0 / scene.num_lights() as f64;
    let (ro, ho) = light.sample_leaving(
        rand_utils::unit_square(),
        rand_utils::unit_square()
    );
    let ng = ho.ng;
    let ns = ho.ns;
    let (pdf_origin, pdf_dir) = light.sample_leaving_pdf(&ro, ng);
    let emit = ho.material.emit(&ho);
    let root = Vertex::light(ho, light, emit, pdf_origin * pdf_light);

    let gathered = emit * ns.dot(ro.dir).abs()
        / (pdf_light * pdf_origin * pdf_dir);

    walk(scene, ro, root, gathered, pdf_dir, Transport::Importance)
}

/// Ray that randomly scatters around from the given root vertex
fn walk<'a>(
    scene: &'a Scene,
    mut ro: Ray,
    root: Vertex<'a>,
    mut gathered: DVec3,
    pdf_dir: f64,
    mode: Transport,
) -> Vec<Vertex<'a>> {
    let mut depth = 0;
    let mut vertices = vec![root];
    let mut pdf_fwd = pdf_dir;
    #[allow(unused_assignments)]
    let mut pdf_bck = 0.0;

    while let Some(ho) = scene.hit(&ro) {
        let material = ho.material;
        gathered *= scene.transmittance(&ho);

        let prev = depth;
        let curr = depth + 1;
        vertices[prev].pdf_fwd = if vertices[prev].is_delta() {
            0.0
        } else {
            let xi = ho.p;
            let ng = ho.ng;
            vertices[prev].solid_angle_to_area(pdf_fwd, xi, ng)
        };
        vertices.push(Vertex::surface(
            ho,
            gathered,
        ));
        let ho = &vertices[curr].h;
        match material.bsdf_pdf(ho, &ro) {
            None => {
                // we hit a light. if tracing from a light, discard latest vertex
                if matches!(mode, Transport::Importance) {
                    vertices.pop();
                }
                break;
            }
            Some(scatter_pdf) => {
                match scatter_pdf.sample_direction(rand_utils::unit_square()) {
                    None => break,
                    Some(wi) => {
                        let xo = ho.p;
                        let wo = ro.dir;
                        let ng = ho.ng;

                        let ns = ho.ns;
                        let ri = ho.generate_ray(wi);
                        // normalized
                        let wi = ri.dir;

                        pdf_fwd = scatter_pdf.value_for(&ri, false);

                        if pdf_fwd <= 0.0 {
                            break;
                        }

                        let shading_cosine = match mode {
                            Transport::Radiance => material.shading_cosine(wi, ns),
                            Transport::Importance => {
                                let xp = vertices[prev].h.p;
                                let v = (xp - xo).normalize();
                                wi.dot(ng).abs() * material.shading_cosine(v, ns)
                                    / v.dot(ng).abs()
                            }
                        };

                        let bsdf = if ho.is_medium() {
                            DVec3::ONE * pdf_fwd
                        } else {
                            material.bsdf_f(wo, wi, mode, &ho)
                        };

                        gathered *= bsdf * shading_cosine / pdf_fwd;

                        pdf_bck = scatter_pdf.value_for(&ri, true);
                        vertices[curr].pdf_bck = if material.is_delta() {
                            0.0
                        } else {
                            let ng = vertices[prev].h.ng;
                            vertices[prev].solid_angle_to_area(pdf_bck, xo, ng)
                        };

                        // russian roulette
                        if depth > 3 {
                            let luminance = crate::rgb_to_luminance(gathered);
                            let rr_prob = (1.0 - luminance).max(0.05);
                            if rand_utils::rand_f64() < rr_prob {
                                break;
                            }

                            // TODO (9)
                            gathered /= 1.0 - rr_prob;
                        }

                        depth += 1;
                        ro = ri;
                    }
                }
            }
        }
    }

    vertices
}
