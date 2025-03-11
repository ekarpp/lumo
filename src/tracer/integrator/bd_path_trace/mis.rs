use super::*;

/// PDF for light leaving from `curr` to `next` w.r.t. surface area
fn pdf_light_leaving(curr: &Vertex, next: &Vertex, scene: &Scene) -> Float {
    if next.is_delta() {
        return 0.0;
    }
    if let Some(light_idx) = curr.light {
        let ho = &curr.h;
        let hi = &next.h;
        let xo = ho.p;
        let xi = hi.p;
        let wi = xi - xo;
        let ri = Ray::new(xo, wi);
        // normalized
        let wi = ri.dir;
        let ng = ho.ng;
        let (light, _) = scene.get_light(light_idx);
        let (_, pdf_dir) = light.sample_leaving_pdf(&ri, ng);
        let ngi = if !next.is_surface() { wi } else { hi.ng };

        // at next
        measure::sa_to_area(pdf_dir, xo, xi, wi, ngi)
    } else {
        0.0
    }
}

/// PDF for direction from camera at `curr` to `next` w.r.t. surface area
fn pdf_camera_leaving(curr: &Vertex, next: &Vertex, camera: &Camera) -> Float {
    if next.is_delta() {
        return 0.0;
    }
    let ho = &curr.h;
    let hi = &next.h;
    let xo = ho.p;
    let xi = hi.p;
    let wi = (xi - xo).normalize();
    let pdf_wi = camera.pdf_wi(&Ray::new(xo, wi));
    let ngi = if !next.is_surface() { wi } else { hi.ng };

    // at next
    measure::sa_to_area(pdf_wi, xo, xi, wi, ngi)
}

/// PDF for starting at `v`
fn pdf_light_origin(v: &Vertex, scene: &Scene) -> Float {
    if let Some(light_idx) = v.light {
        let (light, pdf_light) = scene.get_light(light_idx);
        pdf_light / light.area()
    } else {
        0.0
    }
}

/// Helper to compute updated PDF at the connection. If `prev` is `Some` then it is
/// on a different path from `curr` and `next`. Otherwise `curr` and `next` are from
/// different paths.
fn pdf_connection(curr: &Vertex, next: &Vertex, prev: Option<&Vertex>) -> Float {
    if next.is_delta() {
        return 0.0;
    }
    let ho = &curr.h;
    let hi = &next.h;
    let xo = ho.p;
    let xi = hi.p;

    // p(next|curr,[prev]) w.r.t. SA
    let (pdf_sa, wi) = if let Some(prev) = prev {
        let hp = &prev.h;
        let xp = hp.p;
        let wo = (xp - xo).normalize();
        (curr.bsdf_pdf(wo, true), curr.wo)
    } else {
        let wi = (xi - xo).normalize();
        (curr.bsdf_pdf(wi, false), wi)
    };

    let ngi = if !next.is_surface() { wi } else { hi.ng };

    measure::sa_to_area(pdf_sa, xo, xi, wi, ngi)
}

/// Heuristic applied to PDF in weight computation
pub fn heuristic(ri: Float) -> Float { ri * ri }

/// Computes the MIS weight for the chosen sample strategy. PBRT, what orig paper
pub fn weight(
    scene: &Scene,
    camera: &Camera,
    light_path: &[Vertex],
    camera_path: &[Vertex],
) -> Float {
    let s = light_path.len();
    let t = camera_path.len();
    #[cfg(debug_assertions)]
    {
        assert!(t != 0);
        assert!((t == 1 && s > 1) || t > 1);
    }

    if s + t == 2 {
        return 1.0;
    }

    let map0 = |pdf: Float| {
        if pdf == 0.0 { 1.0 } else { pdf }
    };

    let ct1 = &camera_path[t - 1];
    let ls1 = if s == 0 {
        // if s == 0 ls never gets called but we need a value here
        &camera_path[0]
    } else {
        &light_path[s - 1]
    };

    /* combine light and camera paths to one, starting from the light.
     * collect `pdf_fwd`, `pdf_bck` and `is_delta` to the below vector.
     * modify the values as needed near the connection and compute the weight
     */
    // TODO: drop these
    let mut pdf_rad = Vec::with_capacity(s + t);
    let mut pdf_imp = Vec::with_capacity(s + t);
    let mut is_delta = Vec::with_capacity(s + t);

    // read probabilities from the light path starting from light
    for i in 0..(s.max(2) - 2) {
        pdf_rad.push(light_path[i].pdf_bck);
        pdf_imp.push(light_path[i].pdf_fwd);
        is_delta.push(light_path[i].is_delta());
    }
    // apply updated values, if available, near the connection
    if s > 1 {
        let ls2 = &light_path[s - 2];
        let pdf_bck = pdf_connection(ls1, ls2, Some(ct1));
        pdf_rad.push(pdf_bck);
        pdf_imp.push(ls2.pdf_fwd);
        is_delta.push(ls2.is_delta());
    }
    if s > 0 {
        let pdf_bck = if t == 1 {
            pdf_camera_leaving(ct1, ls1, camera)
        } else {
            pdf_connection(ct1, ls1, None)
        };
        pdf_rad.push(pdf_bck);
        pdf_imp.push(ls1.pdf_fwd);
        is_delta.push(false);
    }
    if t > 0 {
        let pdf_bck = if s == 0 {
            pdf_light_origin(ct1, scene)
        } else if s == 1 {
            pdf_light_leaving(ls1, ct1, scene)
        } else {
            pdf_connection(ls1, ct1, None)
        };
        pdf_rad.push(ct1.pdf_fwd);
        pdf_imp.push(pdf_bck);
        is_delta.push(false);
    }
    if t > 1 {
        let ct2 = &camera_path[t - 2];
        let pdf_bck = if s == 0 {
            pdf_light_leaving(ct1, ct2, scene)
        } else {
            pdf_connection(ct1, ct2, Some(ls1))
        };
        pdf_rad.push(ct2.pdf_fwd);
        pdf_imp.push(pdf_bck);
        is_delta.push(ct2.is_delta());
    }

    // read probabilities on the camera path starting from the end
    for i in (0..(t.max(2) - 2)).rev() {
        pdf_rad.push(camera_path[i].pdf_fwd);
        pdf_imp.push(camera_path[i].pdf_bck);
        is_delta.push(camera_path[i].is_delta());
    }

    #[cfg(test)]
    {
        print!("imp: ");
        for i in 0..pdf_imp.len() {
            print!("{:>9.5} ", pdf_imp[i]);
        }
        println!();
        print!("rad: ");
        for i in 0..pdf_rad.len() {
            print!("{:>9.5} ", pdf_rad[i]);
        }
        println!();
    }

    let mut sum_ri = 0.0;
    let mut ri = 1.0;
    // vertices in light path
    for i in (0..s).rev() {
        ri *= map0(pdf_rad[i]) / map0(pdf_imp[i]);
        #[allow(clippy::nonminimal_bool)]
        if !is_delta[i] && !(i > 0 && is_delta[i - 1]) {
            sum_ri += heuristic(ri);
        }
    }

    ri = 1.0;
    sum_ri += ri;
    // vertices in camera path, skip i = s+t-1, can't hit camera
    for i in s..(s + t - 1) {
        ri *= map0(pdf_imp[i]) / map0(pdf_rad[i]);
        if !is_delta[i] && !is_delta[i + 1] {
            sum_ri += heuristic(ri);
        }
    }

    let weight = 1.0 / sum_ri;
    #[cfg(debug_assertions)]
    if weight < 0.0 {
        println!("negative weight in BDPT MIS");
    }
    weight
}
