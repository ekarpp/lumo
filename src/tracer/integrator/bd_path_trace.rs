use super::*;
use crate::tracer::material::Material;

struct Vertex<'a> {
    h: Hit<'a>,
    gathered: DVec3,
    pdf_next: f64,
    pdf_prev: f64,
}

impl<'a> Vertex<'a> {
    /// Camera vertex
    pub fn camera(xo: DVec3) -> Self {
        let h = Hit::new(
            0.0,
            &Material::Blank,
            xo,
            DVec3::X,
            DVec3::X,
            DVec2::X,
        ).unwrap();
        Self {
            h,
            gathered: DVec3::ONE,
            pdf_prev: 0.0,
            pdf_next: 1.0,
        }
    }

    /// Light vertex
    pub fn light(xo: DVec3, ng: DVec3, gathered: DVec3, pdf_next: f64) -> Self {
        // TODO: need a hit on the sampled point
        let h = Hit::new(
            0.0,
            &Material::Blank,
            xo,
            ng,
            ng,
            DVec2::X,
        ).unwrap();

        Self {
            h,
            gathered,
            pdf_next,
            pdf_prev: 0.0,
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
        }
    }

    pub fn bsdf(&self, prev: &Vertex, next: &Vertex) -> DVec3 {
        // these could be stored already normalized
        let wo = (self.h.p - prev.h.p).normalize();
        let wi = (next.h.p - self.h.p).normalize();

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

    let t = camera_path.len();
    let s = light_path.len();

    if t < 2 || s < 2 {
        // because
        return DVec3::ZERO;
    }

    let light_last = &light_path[s - 1];
    let camera_last = &camera_path[t - 1];

    if !scene.unoccluded(light_last.h.p, camera_last.h.p) {
        DVec3::ZERO
    } else {
        let light_bsdf = light_last.bsdf(&light_path[s - 2], camera_last);
        let camera_bsdf = camera_last.bsdf(&camera_path[t - 2], light_last);

        let illuminance = light_last.gathered * light_bsdf
            * camera_bsdf * camera_last.gathered;

        illuminance * geometry_term(light_last, camera_last)
    }
}

fn geometry_term(v1: &Vertex, v2: &Vertex) -> f64 {
    let v1_xo = v1.h.p;
    let v2_xo = v2.h.p;
    let v1_ng = v1.h.ng;
    let v2_ng = v2.h.ng;

    let wi = (v1_xo - v2_xo).normalize();
    let wi_length_squared = v1_xo.distance_squared(v2_xo);
    // visibility test
    v1_ng.dot(wi).abs() * v2_ng.dot(wi).abs() / wi_length_squared
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
    let emit = DVec3::ONE;//TODO: emit needs a hit light.material().emit(HIT)

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

        let prev_vertex = &mut vertices[depth];

        match material.bsdf_pdf(&ho, &ro) {
            None => break,
            Some(scatter_pdf) => {
                match scatter_pdf.sample_direction(rand_utils::unit_square()) {
                    None => break,
                    Some(wi) => {
                        let ns = ho.ns;
                        let ri = ho.generate_ray(wi);
                        // normalized
                        let wi = ri.dir;

                        let curr_vertex = Vertex::surface(
                            ho,
                            gathered,
                            pdf_next,
                            prev_vertex
                        );

                        pdf_next = scatter_pdf.value_for(&ri);

                        if pdf_next <= 0.0 {
                            break;
                        }

                        gathered *= material.bsdf_f(wo, wi, &curr_vertex.h)
                            * ns.dot(wi).abs()
                            / pdf_next;

                        // TODO: THIS BREAKS FOR REFRACTION IN MICROFACET
                        pdf_prev = pdf_next;
                        prev_vertex.pdf_prev =
                            curr_vertex.solid_angle_to_area(pdf_prev, xo, ng);

                        vertices.push(curr_vertex);

                        // russian roulette
                        if depth > 3 {
                            let luminance = crate::rgb_to_luminance(gathered);
                            let rr_prob = (1.0 - luminance).max(0.05);
                            if rand_utils::rand_f64() < rr_prob {
                                break;
                            }
                            // need to modify vertex PDFs?
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
