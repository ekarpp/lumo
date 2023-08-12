#![allow(warnings)]
use super::*;

// this could use the scoped assignment from PBRT...
/// Computes the MIS weight for the chosen sample strategy. PBRT, what orig paper
pub fn mis_weight(
    camera: &Camera,
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

    let (ls, ct) = if s == 1 {
        (sampled_vertex.as_ref().unwrap(), &camera_path[t - 1])
    } else if t == 1 {
        (&light_path[s - 1], sampled_vertex.as_ref().unwrap())
    } else {
        // max(1) is lazy. if s == 0 ls wont get called anyways though
        (&light_path[s.max(1) - 1], &camera_path[t - 1])
    };

    let mut sum_ri = 0.0;
    let mut ri = 1.0;

    // applies the updated PDF for camera_last of the connection
    if t > 0 {
        let pdf_prev = if s == 0 {
            ct.pdf_light_origin()
        } else if s == 1 {
            ls.pdf_light_leaving(ct)
        } else {
            let ls_m = &light_path[s - 2];
            ls.pdf_area(ls_m, ct, Transport::Importance)
        };

        ri *= map0(pdf_prev) / map0(ct.pdf_fwd);
        sum_ri += ri;
    }

    // applies the updated PDF for camera t - 2 using the connection
    if t > 1 {
        let ct_m = &camera_path[t - 2];
        let pdf_prev = if s == 0 {
            ct.pdf_light_leaving(ct_m)
        } else {
            ct.pdf_area(ls, ct_m, Transport::Importance)
        };
        ri *= map0(pdf_prev) / map0(ct_m.pdf_fwd);
        if !ct_m.is_delta() {
            sum_ri += ri;
        }
    }

    // vertices in camera path
    for i in (1..t.max(2) - 2).rev() {
        ri *= map0(camera_path[i].pdf_bck) / map0(camera_path[i].pdf_fwd);
        if !camera_path[i].is_delta() && !camera_path[i - 1].is_delta() {
            sum_ri += ri;
        }
    }

    let mut ri = 1.0;

    // applies the updated PDF at light_last using the connection
    if s > 0 {
        let pdf_prev = if t == 1 {
            // move this to vertex
            let xo = ct.h.p;
            let xi = ls.h.p;
            let wi = (xi - xo).normalize();
            camera.pdf(wi)
        } else {
            let ct_m = &camera_path[t - 2];
            ct.pdf_area(ct_m, ls, Transport::Radiance)
        };
        ri *= map0(pdf_prev) / map0(ls.pdf_fwd);
        sum_ri += ri;
    }

    // applies the updated PDF at light_last using the connection
    if s > 1 {
        let ls_m = &light_path[s - 2];
        let pdf_prev = ls.pdf_area(ct, ls_m, Transport::Radiance);

        ri *= map0(pdf_prev) / map0(ls_m.pdf_fwd);
        if !ls_m.is_delta() {
            sum_ri += ri;
        }
    }

    // vertices in light path
    for i in (0..s.max(2) - 2).rev() {
        ri *= map0(light_path[i].pdf_bck) / map0(light_path[i].pdf_fwd);
        if !light_path[i].is_delta() && !light_path[(i - 1).max(0)].is_delta() {
            sum_ri += ri;
        }
    }

    let weight = 1.0 / (1.0 + sum_ri);
    #[cfg(debug_assertions)]
    if weight < 0.0 {
        println!("negative weight in BDPT MIS");
    }
    weight
}
