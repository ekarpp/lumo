#![allow(clippy::manual_swap)]
use super::*;
use crate::tracer::{
    bxdf::BxDF, bsdf::BSDF, CameraBuilder, Spectrum,
    Instanceable, Sphere, Texture
};

const NUM_PATHS: usize = 10_000;

fn camera() -> Camera {
    Camera::builder().build()
}

fn scene() -> Scene {
    Scene::empty_box(
        Spectrum::WHITE,
        Material::diffuse(Texture::from(Spectrum::RED)),
        Material::Standard(BSDF::new(BxDF::Lambertian(Spectrum::GREEN)))
    )
}

#[test]
fn all_sum_to_one_diffuse() {
    let sce = scene();
    test_scene(sce, camera())
}

#[test]
fn all_sum_to_one_medium() {
    let mut sce = scene();
    sce.set_medium(
        crate::tracer::Medium::new(
            Vec3::new(0.002, 0.003, 0.0001),
            Vec3::new(0.175, 0.125, 0.11),
            0.9,
        )
    );
    test_scene(sce, camera())
}

#[test]
fn all_sum_to_one_specular_delta() {
    let mut sce = scene();

    sce.add(Sphere::new(
        0.25,
        Material::mirror())
            .translate(-0.45, -0.5, -1.5)
    );
    sce.add(Sphere::new(
        0.25,
        Material::glass())
            .translate(0.45, -0.5, -1.3)
    );

    test_scene(sce, camera())
}

#[test]
fn all_sum_to_one_specular_rough() {
    let mut sce = scene();

    sce.add(Sphere::new(
        0.25,
        Material::metal(Texture::from(Spectrum::WHITE), 0.5, 1.5, 1.5))
            .translate(-0.45, -0.5, -1.5)
    );
    sce.add(Sphere::new(
        0.25,
        Material::transparent(Texture::from(Spectrum::WHITE), 0.5, 1.5))
            .translate(0.45, -0.5, -1.3)
    );

    test_scene(sce, camera())
}

#[test]
fn all_sum_to_one_big_scale() {
    let sce = Scene::cornell_box();
    let cam = CameraBuilder::new()
        .origin(Vec3::new(278.0, 273.0, -800.0))
        .towards(Vec3::new(278.0, 273.0, 0.0))
        .zoom(2.8)
        .focal_length(0.035)
        .resolution((512, 512))
        .build();
    test_scene(sce, cam)
}

fn test_scene(sce: Scene, cam: Camera) {
    let r = cam.generate_ray(Vec2::ZERO);
    let xc = r.origin;
    let lambda = ColorWavelength::sample(rand_utils::rand_float());
    for _ in 0..NUM_PATHS {
        let mut pth;
        loop {
            pth = path_gen::light_path(&sce, &lambda);
            // too short, try another path
            if pth.len() <= 2 { continue; }
            let ls = &pth[pth.len() - 1];
            // last is delta, try another path
            if ls.is_delta() { continue; }

            let ls_m = &pth[pth.len() - 2];
            let ho = &ls_m.h;
            let xo = ho.p;
            let wo = ls.wo;
            let ngo = ho.ng;

            let hi = &ls.h;
            let ngi = hi.ng;
            let xi = hi.p;

            let t2 = xi.distance_squared(xc);
            let wi = (xi - xc).normalize();
            let ri = Ray::new(xc, wi);
            // can't hit last from camera, try another path
            if sce.hit(&ri).is_some_and(|h: Hit| h.t * h.t < t2 - crate::EPSILON.powi(2)) {
                continue;
            }
            // good path found, append camera vertex and update
            pth.push(Vertex::camera(
                xc,
                0.0,
                Color::WHITE
            ));

            let len = pth.len();
            pth[len - 1].wo = wi;

            // update ls connection pdf with added camera vertex
            let pdf_sa = cam.pdf_wi(&ri);
            let ngi = if !pth[len - 2].is_surface() { wi } else { ngi };
            pth[len - 2].pdf_bck = measure::sa_to_area(pdf_sa, xc, xi, wi, ngi);

            // update ls_m connection pdf with added camera vertex
            if !pth[len - 3].is_delta() {
                let pdf_sa = pth[len - 2].bsdf_pdf(-wi, true);
                let ngo = if !pth[len - 3].is_surface() { wo } else { ngo };
                pth[len - 3].pdf_bck = measure::sa_to_area(pdf_sa, xo, xi, wo, ngo);
            }

            break;
        }

        let map0 = |pdf: Float| {
            if pdf == 0.0 { 1.0 } else { pdf }
        };

        let lp = pth.clone();
        let mut cp: Vec<Vertex> = pth.clone().into_iter().rev().collect();

        for i in (1..cp.len()).rev() {
            cp[i].wo = -cp[i - 1].wo;
        }
        cp[0].wo = Vec3::ZERO;
        print!("path type: ");
        for i in 0..cp.len() {
            print!("{}", if cp[i].is_delta() { 'd' }
                   else if cp[i].is_surface() { 's' }
                   else { 'm' }
            );
            let tmp = cp[i].pdf_fwd;
            cp[i].pdf_fwd = cp[i].pdf_bck;
            cp[i].pdf_bck = tmp;
        }
        println!();
        print!("imp: ");
        for i in 0..lp.len() {
            print!("{:>9.5} ", lp[i].pdf_fwd);
        }
        println!();
        print!("rad: ");
        for i in 0..lp.len() {
            print!("{:>9.5} ", lp[i].pdf_bck);
        }
        println!();

        let mut pis = vec!();
        let mut wis = vec!();
        let mut sump = 0.0;
        let mut sumw = 0.0;

        for s in 0..pth.len() {
            let t = pth.len() - s;
            if t == 1 && s < 2 { continue; }

            if pth[s].is_delta() || (s > 0 && pth[s - 1].is_delta()) {
                continue;
            }

            println!("s = {:>2}", s);
            let mut pi = 1.0;
            for i in 0..s {
                if !pth[i].is_delta() {
                    pi *= map0(pth[i].pdf_fwd);
                }
            }
            for i in s..(s + t - 1) {
                if !pth[i].is_delta() {
                    pi *= map0(pth[i].pdf_bck);
                }
            }
            pi = mis::heuristic(pi);
            pis.push((s, pi));
            sump += pi;

            let sampled = if t == 1 {
                Some( cp[0].clone() )
            } else if s == 1 {
                Some( lp[0].clone() )
            } else {
                None
            };
            let wi = mis::weight(&cam, &lp, s, &cp, t, sampled);
            wis.push(wi);
            sumw += wi;
        }

        for i in 0..pis.len() {
            let (s, pi) = pis[i];
            let wi = pi / sump;
            println!("s = {:>2}: {:.5} {:.5} (err: {:+.5})", s, wi, wis[i], wi - wis[i]);
        }
        println!();
        println!("{}", sumw);
        assert!((1.0 - sumw).abs() < 0.01);
    }
}
