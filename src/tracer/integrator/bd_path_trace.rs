use super::*;

struct Vertex {
    ng: DVec3,
    xo: DVec3,
    gathered: DVec3,
    pdf_next: f64,
    pdf_prev: f64,
}

impl Vertex {
    /// Camera vertex
    pub fn camera(xo: DVec3) -> Self {
        Self {
            xo,
            ng: DVec3::Z,
            gathered: DVec3::ONE,
            pdf_prev: 0.0,
            pdf_next: 1.0,
        }
    }

    /// Light vertex
    pub fn light(xo: DVec3, ng: DVec3, gathered: DVec3, pdf_next: f64) -> Self {
        Self {
            xo,
            ng,
            gathered,
            pdf_next,
            pdf_prev: 0.0,
        }
    }

    /// Surface vertex
    pub fn surface(
        xo: DVec3,
        ng: DVec3,
        gathered: DVec3,
        pdf_next: f64,
        prev: &Vertex
    ) -> Self {
        Self {
            xo,
            ng,
            gathered,
            pdf_next: prev.solid_angle_to_area(pdf_next, xo, ng),
            pdf_prev: 0.0,
        }
    }

    fn solid_angle_to_area(&self, pdf: f64, xo: DVec3, ng: DVec3) -> f64 {
        let wi = (xo - self.xo).normalize();
        pdf * wi.dot(ng).abs() / xo.distance_squared(self.xo)
    }
}

pub fn integrate(scene: &Scene, r: Ray) -> DVec3 {
    let light_path = light_path(scene);
    let camera_path = camera_path(scene, r);
    DVec3::ZERO
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

fn walk(
    scene: &Scene,
    mut ro: Ray,
    root: Vertex,
    mut gathered: DVec3,
    pdf_dir: f64,
    from_light: bool,
) -> Vec<Vertex> {
    let mut depth = 0;
    let mut vertices = vec![root];
    let mut pdf_next = pdf_dir;
    let mut pdf_prev = 0.0;

    while let Some(ho) = scene.hit(&ro) {
        let material = ho.object.material();
        let xo = ho.p;
        let ng = ho.ng;

        let prev_vertex = &mut vertices[depth];
        let curr_vertex = Vertex::surface(xo, ng, gathered, pdf_next, prev_vertex);


        match material.bsdf_pdf(&ho, &ro) {
            None => break,
            Some(scatter_pdf) => {
                match scatter_pdf.sample_direction(rand_utils::unit_square()) {
                    None => break,
                    Some(wi) => {
                        let ri = ho.generate_ray(wi);
                        // normalized
                        let wi = ri.dir;
                        pdf_next = scatter_pdf.value_for(&ri);

                        if pdf_next <= 0.0 {
                            break;
                        }

                        let ns = ho.ns;

                        gathered *= material.bsdf_f(&ro, &ri, &ho)
                            * ns.dot(wi).abs()
                            / pdf_next;

                        pdf_prev = todo!();
                        prev_vertex.pdf_prev =
                            curr_vertex.solid_angle_to_area(pdf_prev, xo, ng);

                        vertices.push(curr_vertex);
                        depth += 1;
                        ro = ri;
                    }
                }
            }
        }
    }

    vertices
}
