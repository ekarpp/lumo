use super::*;

struct Vertex {
    ng: DVec3,
    xo: DVec3,
    gathered: DVec3,
    pdf_light: f64,
    pdf_camera: f64,
}

impl Vertex {
    pub fn camera(xo: DVec3) -> Self {
        Self {
            xo,
            ng: DVec3::Z,
            gathered: DVec3::ONE,
            pdf_light: 0.0,
            pdf_camera: 1.0,
        }
    }

    pub fn light(xo: DVec3, ng: DVec3, gathered: DVec3, pdf_light: f64) -> Self {
        Self {
            xo,
            ng,
            gathered,
            pdf_light,
            pdf_camera: 0.0,
        }
    }
}

pub fn integrate(s: &Scene, r: Ray) -> DVec3 {
    let light_path = light_path(s);
    let camera_path = camera_path(s, r);
    DVec3::ZERO
}

fn camera_path(s: &Scene, r: Ray) -> Vec<Vertex> {
    let root = Vertex::camera(r.origin);
    let gathered = DVec3::ONE;

    walk(s, r, root, gathered, false)
}

fn light_path(s: &Scene) -> Vec<Vertex> {
    let light = s.uniform_random_light();
    let pdf_light = 1.0 / s.num_lights() as f64;
    let (ro, ng) = light.sample_leaving(
        rand_utils::unit_square(),
        rand_utils::unit_square()
    );
    let (pdf_origin, pdf_dir) = light.sample_leaving_pdf(&ro, ng);
    let emit = DVec3::ONE;//TODO: emit needs a hit light.material().emit(HIT)

    let root = Vertex::light(ro.origin, ng, emit, pdf_origin * pdf_light);

    let gathered = emit * ng.dot(ro.dir).abs()
        / (pdf_light * pdf_origin * pdf_dir);

    walk(s, ro, root, gathered, true)
}

fn walk(
    s: &Scene,
    r: Ray,
    root: Vertex,
    gathered: DVec3,
    from_light: bool
) -> Vec<Vertex> {
    vec![root]
}
