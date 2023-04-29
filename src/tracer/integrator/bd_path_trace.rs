use super::*;
use crate::tracer::material::Material;

/*
 * TODO:
 * (1, 7) hit for light vertex initialization
 * (2) store directions in vertex?
 * (3, 6) proper visibility test
 * (4) PBRT has no geometry term but we need it?
 * (5) implement MIS
 * (8) previous pdf needs pdf with orders swapped. refraction not commutative
 * (9) need to modify vertex PDFs?
 *
 * + this needs proper refactoring and cleanup...
*/

struct Vertex<'a> {
    h: Hit<'a>,
    gathered: DVec3,
    pdf_next: f64,
    pdf_prev: f64,
    pub on_light: bool,
}

impl<'a> Vertex<'a> {
    /// Camera vertex
    pub fn camera(xo: DVec3) -> Self {
        let h = Hit::new(
            0.0,
            &Material::Blank,
            xo,
            DVec3::ZERO,
            DVec3::X,
            DVec3::X,
            DVec2::X,
        ).unwrap();
        Self {
            h,
            gathered: DVec3::ONE,
            pdf_prev: 0.0,
            pdf_next: 1.0,
            on_light: false,
        }
    }

    /// Light vertex
    pub fn light(xo: DVec3, ng: DVec3, gathered: DVec3, pdf_next: f64) -> Self {
        // TODO (1)
        let h = Hit::new(
            0.0,
            &Material::Blank,
            xo,
            DVec3::ZERO,
            ng,
            ng,
            DVec2::X,
        ).unwrap();

        Self {
            h,
            gathered,
            pdf_next,
            pdf_prev: 0.0,
            on_light: true,
        }
    }

    /// Surface vertex
    pub fn surface(
        h: Hit<'a>,
        gathered: DVec3,
        pdf_next: f64,
        prev: &Vertex
    ) -> Self {
        let xo = h.p;
        let ng = h.ng;
        Self {
            h,
            gathered,
            pdf_next: prev.solid_angle_to_area(pdf_next, xo, ng),
            pdf_prev: 0.0,
            on_light: false,
        }
    }

    pub fn bsdf(&self, prev: &Vertex, next: &Vertex) -> DVec3 {
        // TODO (2)
        let wo = (self.h.p - prev.h.p).normalize();
        let wi = (next.h.p - self.h.p).normalize();

        // TODO (3)
        // this is a dumb hack.
        // need to somehow figure if prev and next are on the same object.
        // what if the object is transparent?
        if wi.dot(self.h.ng) < crate::EPSILON {
            DVec3::ZERO
        } else {
            self.h.material.bsdf_f(wo, wi, &self.h)
        }
    }

    fn solid_angle_to_area(&self, pdf: f64, xo: DVec3, ng: DVec3) -> f64 {
        let wi = (xo - self.h.p).normalize();
        pdf * wi.dot(ng).abs() / xo.distance_squared(self.h.p)
    }
}

pub fn integrate(scene: &Scene, r: Ray) -> DVec3 {
    let light_path = light_path(scene);
    let camera_path = camera_path(scene, r);

    let mut radiance = DVec3::ZERO;

    for t in 0..=camera_path.len() {
        for s in 0..=light_path.len() {
            radiance +=
                connect_paths(scene, &light_path[0..s], &camera_path[0..t]);
        }
    }

    radiance
}

fn connect_paths(scene: &Scene, light_path: &[Vertex], camera_path: &[Vertex]) -> DVec3 {
    let t = camera_path.len();
    let s = light_path.len();

    let mut sampled_vertex: Option<Vertex> = None;

    let radiance = if t == 0 || t == 1 {
        // we dont do camera sampling
        DVec3::ZERO
    } else if s == 0 {
        let camera_last = &camera_path[t - 1];
        if !camera_last.on_light {
            DVec3::ZERO
        } else {
            let emittance = camera_last.h.material.emit(&camera_last.h);
            camera_last.gathered * emittance
        }
    } else if s == 1 {
        let camera_last = &camera_path[t - 1];
        if camera_last.on_light || camera_last.h.material.is_delta() {
            DVec3::ZERO
        } else {
            // sample a point on the light
            let light = scene.uniform_random_light();

            let xo = camera_last.h.p;
            let pdf_light = ObjectPdf::new(light, xo);

            match pdf_light.sample_direction(rand_utils::unit_square()) {
                None => DVec3::ZERO,
                Some(wi) => {
                    let ri = camera_last.h.generate_ray(wi);
                    match scene.hit_light(&ri, light) {
                        None => DVec3::ZERO,
                        Some(hi) => {
                            let xi = hi.p;
                            let ng = hi.ng;
                            let ns = hi.ns;
                            let emittance = hi.material.emit(&hi);
                            let light_last = Vertex::light(
                                xi,
                                ng,
                                emittance,
                                0.0,
                            );
                            let bsdf = camera_last.bsdf(
                                &camera_path[t - 2],
                                &light_last
                            );

                            let g = geometry_term(&light_last, camera_last);
                            sampled_vertex = Some(light_last);
                            // TODO (4)
                            // geometry term not used in PBRT, but it breaks w/o
                            camera_last.gathered * bsdf
                                * ns.dot(wi).abs() * emittance
                                * g
                        }
                    }
                }
            }
        }
    } else {
        let light_last = &light_path[s - 1];
        let camera_last = &camera_path[t - 1];

        if camera_last.on_light || !scene.unoccluded(light_last.h.p, camera_last.h.p) {
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
        mis_weight(scene, light_path, camera_path, sampled_vertex)
    };
    // do MIS weight for path
    radiance * weight
}

fn mis_weight(
    _scene: &Scene,
    _light_path: &[Vertex],
    _camera_path: &[Vertex],
    _sampled_vertex: Option<Vertex>
) -> f64 {
    // TODO (5)
    1.0
}

fn geometry_term(v1: &Vertex, v2: &Vertex) -> f64 {
    let v1_xo = v1.h.p;
    let v2_xo = v2.h.p;
    let v1_ns = v1.h.ns;
    let v2_ns = v2.h.ns;

    let wi = (v1_xo - v2_xo).normalize();
    let wi_length_squared = v1_xo.distance_squared(v2_xo);
    // TODO (6) proper visibility test, here?
    v1_ns.dot(wi).abs() * v2_ns.dot(wi).abs() / wi_length_squared
}

fn camera_path(scene: &Scene, r: Ray) -> Vec<Vertex> {
    let root = Vertex::camera(r.origin);
    let gathered = DVec3::ONE;

    walk(scene, r, root, gathered, 1.0, false)
}

fn light_path(scene: &Scene) -> Vec<Vertex> {
    let light = scene.uniform_random_light();
    let pdf_light = 1.0 / scene.num_lights() as f64;
    let (ro, ng) = light.sample_leaving(
        rand_utils::unit_square(),
        rand_utils::unit_square()
    );
    let (pdf_origin, pdf_dir) = light.sample_leaving_pdf(&ro, ng);
    // TODO (7) Need the hit also here to get emittance
    let emit = DVec3::ONE;

    let root = Vertex::light(ro.origin, ng, emit, pdf_origin * pdf_light);

    let gathered = emit * ng.dot(ro.dir).abs()
        / (pdf_light * pdf_origin * pdf_dir);

    walk(scene, ro, root, gathered, pdf_dir, true)
}

fn walk<'a>(
    scene: &'a Scene,
    mut ro: Ray,
    root: Vertex<'a>,
    mut gathered: DVec3,
    pdf_dir: f64,
    from_light: bool,
) -> Vec<Vertex<'a>> {
    let mut depth = 0;
    let mut vertices = vec![root];
    let mut pdf_next = pdf_dir;
    let mut pdf_prev = 0.0;

    while let Some(ho) = scene.hit(&ro) {
        let material = ho.material;
        let xo = ho.p;
        let wo = ro.dir;
        let ng = ho.ng;

        let prev = depth;
        let curr = depth + 1;
        vertices.push(Vertex::surface(
            ho,
            gathered,
            pdf_next,
            &vertices[prev],
        ));
        let ho = &vertices[curr].h;
        match material.bsdf_pdf(ho, &ro) {
            None => {
                vertices[curr].on_light = true;
                break;
            }
            Some(scatter_pdf) => {
                match scatter_pdf.sample_direction(rand_utils::unit_square()) {
                    None => break,
                    Some(wi) => {
                        let ns = ho.ns;
                        let ri = ho.generate_ray(wi);
                        // normalized
                        let wi = ri.dir;

                        pdf_next = scatter_pdf.value_for(&ri);

                        if pdf_next <= 0.0 {
                            break;
                        }

                        gathered *= material.bsdf_f(wo, wi, &ho)
                            * ns.dot(wi).abs()
                            / pdf_next;

                        // TODO (8)
                        pdf_prev = pdf_next;
                        vertices[prev].pdf_prev =
                            vertices[curr].solid_angle_to_area(pdf_prev, xo, ng);

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
