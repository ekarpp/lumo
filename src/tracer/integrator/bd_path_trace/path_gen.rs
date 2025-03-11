use super::*;

/// Generates a ray path starting from the camera
pub fn camera_path<'a>(
    scene: &'a Scene,
    camera: &'a Camera,
    r: Ray,
    rng: &mut Xorshift,
    lambda: &ColorWavelength,
) -> Vec<Vertex<'a>> {
    let gathered = Color::WHITE;
    let xo = r.origin;
    let pdf_wi = camera.pdf_wi(&r);
    let pdf_xo = camera.pdf_xo(&r);
    let root = Vertex::camera(xo, pdf_xo, gathered);

    walk(scene, r, rng, lambda, root, gathered, pdf_wi, Transport::Radiance)
}

/// Generates a ray path strating from a light
pub fn light_path<'a>(
    scene: &'a Scene,
    rng: &mut Xorshift,
    lambda: &'a ColorWavelength
) -> Vec<Vertex<'a>> {
    let light_idx = scene.sample_light(rng.gen_float());
    let (light, pdf_light) = scene.get_light(light_idx);
    let (ri, ho) = light.sample_leaving(
        rng.gen_vec2(),
        rng.gen_vec2(),
    );
    let ng = ho.ng;
    let ns = ho.ns;
    let (pdf_origin, pdf_dir) = light.sample_leaving_pdf(&ri, ng);
    let emit = ho.material.emit(lambda, &ho);
    let root = Vertex::light(
        ho,
        light_idx,
        emit,
        pdf_origin * pdf_light
    );

    let wi = ri.dir;
    let gathered = emit * wi.dot(ns).abs() / (pdf_light * pdf_origin * pdf_dir);

    walk(scene, ri, rng, lambda, root, gathered, pdf_dir, Transport::Importance)
}

/// Ray that randomly scatters around from the given root vertex
#[allow(clippy::too_many_arguments)]
fn walk<'a>(
    scene: &'a Scene,
    mut ro: Ray,
    rng: &mut Xorshift,
    lambda: &ColorWavelength,
    root: Vertex<'a>,
    mut gathered: Color,
    pdf_dir: Float,
    mode: Transport,
) -> Vec<Vertex<'a>> {
    let mut depth = 0;
    let mut vertices = Vec::with_capacity(8);
    vertices.push(root);
    // w.r.t. SA
    let mut pdf_fwd = pdf_dir;

    while let Some(ho) = scene.hit(&ro, rng) {
        let material = ho.material;
        gathered *= scene.transmittance(lambda, ho.t);

        let prev = depth;
        let wo = -ro.dir;
        vertices.push(Vertex::surface(
            wo,
            ho,
            gathered,
            pdf_fwd,
            &vertices[prev],
        ));
        depth += 1;
        let curr = depth;
        let ho = &vertices[curr].h;

        match material.bsdf_sample(wo, ho, rng.gen_float(), rng.gen_vec2()) {
            None => {
                /* we hit a light.
                 * if tracing from a light, discard latest vertex.
                 * if tracing from a camera, update light pointer.
                 */
                if matches!(mode, Transport::Importance) {
                    vertices.pop();
                } else {
                    vertices[curr].light = scene.get_light_at(ho);
                }
                break;
            }
            Some(wi) => {
                let ri = ho.generate_ray(wi);
                let wi = ri.dir;

                pdf_fwd = material.bsdf_pdf(wo, wi, ho, false);

                if pdf_fwd == 0.0 {
                    break;
                }

                // correction term for shading cosine due to non-symmetry
                let shading_correction = match mode {
                    Transport::Radiance => 1.0,
                    Transport::Importance => vertices[curr].shading_correction(wi),
                };

                let bsdf = material.bsdf_f(wo, wi, lambda, mode, ho);
                let bsdf = if ho.is_medium() {
                    bsdf * pdf_fwd
                } else {
                    bsdf
                };

                gathered *= bsdf
                    * vertices[curr].shading_cosine(wi)
                    * shading_correction
                    / pdf_fwd;

                // only MIS cares about this
                vertices[prev].pdf_bck = vertices[curr].pdf_prev(&vertices[prev], wi);

                if depth >= RR_DEPTH {
                    let luminance = gathered.luminance(lambda);
                    let rr_prob = (1.0 - luminance).max(RR_MIN);
                    if rng.gen_float() < rr_prob {
                        break;
                    }
                    gathered /= 1.0 - rr_prob;
                }

                if material.is_delta() {
                    pdf_fwd = 0.0;
                }

                ro = ri;
            }
        }
    }

    vertices
}
