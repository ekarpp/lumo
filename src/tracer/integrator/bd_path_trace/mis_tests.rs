use super::*;
use crate::tracer::{
    RGB, CameraBuilder, Spectrum,
    Instanceable, Sphere, Texture, CameraType
};

const NUM_PATHS: usize = 10_000;

fn camera() -> Camera {
    Camera::builder().build()
}

fn scene() -> Scene {
    Scene::empty_box(
        Spectrum::WHITE,
        Material::diffuse(Texture::from(Spectrum::RED)),
        Material::lambertian(Spectrum::GREEN),
    )
}

#[test]
fn all_sum_to_one_diffuse() {
    let sce = scene();
    test_scene(sce, camera())
}

#[test]
fn all_sum_to_one_orthographic() {
    let sce = scene();
    let cam = Camera::builder()
        .camera_type(CameraType::Orthographic)
        .build();

    test_scene(sce, cam)
}

#[test]
fn all_sum_to_one_medium() {
    let mut sce = scene();
    sce.set_medium(
        crate::tracer::Medium::new(
            RGB::from(Vec3::new(0.002, 0.003, 0.0001)),
            RGB::from(Vec3::new(0.175, 0.125, 0.11)),
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
        .origin(278.0, 273.0, -800.0)
        .towards(278.0, 273.0, 0.0)
        .zoom(2.8)
        .focal_length(0.035)
        .resolution((512, 512))
        .build();
    test_scene(sce, cam)
}

fn test_scene(mut sce: Scene, cam: Camera) {
    sce.build();
    let mut rng = Xorshift::default();

    let map0 = |pdf: Float| {
        if pdf == 0.0 { 1.0 } else { pdf }
    };

    for i in 0..NUM_PATHS {
        let lambda = ColorWavelength::sample(rng.gen_float());
        let (cp, lp) = if i % 2 == 0 {
            _from_light(&mut rng, &lambda, &sce, &cam)
        } else {
            _from_camera(&mut rng, &lambda, &sce, &cam)
        };

        let mut pis = vec!();
        let mut wis = vec!();
        let mut sump = 0.0;
        let mut sumw = 0.0;

        for s in 0..lp.len() {
            let t = lp.len() - s;
            if t == 1 && s < 2 { continue; }

            if lp[s].is_delta() || (s > 0 && lp[s - 1].is_delta()) {
                continue;
            }

            println!("s = {:>2}", s);
            let mut pi = 1.0;
            for i in 0..s {
                if !lp[i].is_delta() {
                    pi *= map0(lp[i].pdf_fwd);
                }
            }
            for i in s..(s + t - 1) {
                if !lp[i].is_delta() {
                    pi *= map0(lp[i].pdf_bck);
                }
            }
            pi = mis::heuristic(pi);
            pis.push((s, pi));
            sump += pi;

            let wi = mis::weight(&sce, &cam, &lp[..s], &cp[..t]);
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


fn _from_camera<'a>(
    rng: &'a mut Xorshift,
    lambda: &'a ColorWavelength,
    sce: &'a Scene,
    cam: &'a Camera
) -> (Vec<Vertex<'a>>, Vec<Vertex<'a>>) {
    let mut pth;

    let res = Vec2::new(
        cam.get_resolution().x as Float,
        cam.get_resolution().y as Float,
    );

    let min_len = 3;

    'outer: loop {
        let ro = cam.generate_ray(res * rng.gen_vec2(), rng.gen_vec2());
        pth = path_gen::camera_path(sce, cam, ro, rng, lambda);
        let ct = &pth[pth.len() - 1];
        if pth.len() < min_len { continue; }
        if ct.is_light() { break; }

        pth.push(Vertex::camera(Vec3::ZERO, 0.0, Color::BLACK));
        while pth.len() > min_len - 1 {
            pth.remove(pth.len() - 1);
            let ct = &pth[pth.len() - 1];
            if ct.is_delta() { continue; }
            let ho = &ct.h;
            let xo = ho.p;

            let light_idx = sce.sample_light(rng.gen_float());
            let (light, _) = sce.get_light(light_idx);
            let xi = light.sample_towards(xo, rng.gen_vec2());
            let wi = (xi - xo).normalize();
            let r = ho.generate_ray(wi);

            let hi = sce.hit_light(&r, rng, light);
            if hi.is_none() { continue; }

            let pdf_sa = ct.bsdf_pdf(wi, false);
            if pdf_sa == 0.0 { continue; }

            let mut vert = Vertex::surface(-wi, hi.unwrap(), Color::WHITE, pdf_sa, ct);
            vert.light = Some(light_idx);
            pth.push(vert);
            break 'outer;
        }
    }

    let len = pth.len();
    let ct = &pth[len - 1];
    let ct_m = &pth[len - 2];
    let ct_mm = &pth[len - 3];

    let light_idx = ct.light.unwrap();
    let (light, light_pdf) = sce.get_light(light_idx);

    let ho = &ct_m.h;
    let xo = ho.p;

    let hi = &ct.h;
    let xi = hi.p;

    let wi = (xi - xo).normalize();

    let hp = &ct_mm.h;
    let xp = hp.p;

    let ngi = hi.ng;
    let ngo = ho.ng;
    let ngp = hp.ng;

    let rl = Ray::new(xi, -wi);

    let (pdf_origin, pdf_dir) = light.sample_leaving_pdf(&rl, ngi);

    pth[len - 1].pdf_bck = light_pdf * pdf_origin;

    if !pth[len - 2].is_delta() {
        let ngo = if !pth[len - 2].is_surface() { -wi } else { ngo };
        pth[len - 2].pdf_bck = measure::sa_to_area(pdf_dir, xi, xo, -wi, ngo);
    }

    if !pth[len - 3].is_delta() {
        let pdf_sa = pth[len - 2].bsdf_pdf(wi, true);
        let wo = pth[len - 2].wo;
        let ngp = if !pth[len - 3].is_surface() { wo } else { ngp };
        pth[len - 3].pdf_bck = measure::sa_to_area(pdf_sa, xo, xp, wo, ngp);
    }

    let cp = pth;
    println!("from camera");
    let lp = _reverse(cp.clone());

    (cp, lp)
}

fn _from_light<'a>(
    rng: &'a mut Xorshift,
    lambda: &'a ColorWavelength,
    sce: &'a Scene,
    cam: &'a Camera
) -> (Vec<Vertex<'a>>, Vec<Vertex<'a>>) {
    let mut pth;
    let r = cam.generate_ray(Vec2::ZERO, rng.gen_vec2());
    let xc = r.origin;
    loop {
        pth = path_gen::light_path(sce, rng, lambda);
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
        if sce.hit(&ri, rng).is_some_and(|h: Hit| h.t * h.t < t2 - crate::EPSILON.powi(2)) {
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

    let lp = pth;
    println!("from light");
    let cp = _reverse(lp.clone());
    (cp, lp)
}

#[allow(clippy::manual_swap)]
fn _reverse(mut pth: Vec<Vertex>) -> Vec<Vertex> {
    pth.reverse();
    for i in (1..pth.len()).rev() {
        pth[i].wo = -pth[i - 1].wo;
    }
    pth[0].wo = Vec3::ZERO;
    print!("path type: ");
    for i in 0..pth.len() {
        print!("{}", if pth[i].is_delta() { 'd' }
               else if pth[i].is_surface() { 's' }
               else { 'm' }
        );
        let tmp = pth[i].pdf_fwd;
        pth[i].pdf_fwd = pth[i].pdf_bck;
        pth[i].pdf_bck = tmp;
    }
    println!();
    print!("imp: ");
    for i in 0..pth.len() {
        print!("{:>9.5} ", pth[i].pdf_fwd);
    }
    println!();
    print!("rad: ");
    for i in 0..pth.len() {
        print!("{:>9.5} ", pth[i].pdf_bck);
    }
    println!();

    pth
}
