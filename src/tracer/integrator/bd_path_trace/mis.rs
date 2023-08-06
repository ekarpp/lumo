#![allow(warnings)]
use super::*;

/// Computes the MIS weight for the chosen sample strategy. PBRT, what orig paper
pub fn mis_weight(
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
            if ct.is_delta() {
                0.0
            } else {
                ct.h.light.map_or(0.0, |light| 1.0 / light.area())
            }
            // check camera and light path initializations. fix the mis weights to be like there. double check they are correct.
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
            ls.pdf_area(ls_m, ct, Transport::Importance)
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
            ct.pdf_area(ls, ct_m, Transport::Importance)
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
            ct.pdf_area(ct_m, ls, Transport::Radiance)
        };
        ri *= map0(pdf_prev) / map0(ls.pdf_fwd);
        sum_ri += ri;
    }

    // applies the updated PDF at light_last using the connection
    if s > 1 {
        let ls = &light_path[s - 1];
        let ls_m = &light_path[s - 2];
        let ct = sampled_vertex.as_ref().unwrap_or(&camera_path[t - 1]);

        let pdf_prev = ls.pdf_area(ct, ls_m, Transport::Radiance);

        ri *= map0(pdf_prev) / map0(ls_m.pdf_fwd);
        sum_ri += ri;
    }

    // vertices in light path
    for i in (0..s.max(2) - 2).rev() {
        ri *= map0(light_path[i].pdf_bck) / map0(light_path[i].pdf_fwd);
        sum_ri += ri;
    }

    let weight = 1.0 / (1.0 + sum_ri);
    #[cfg(debug_assertions)]
    if weight < 0.0 {
        println!("negative weight in BDPT MIS");
    }
    weight
}
