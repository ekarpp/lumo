use super::*;

const NUM_SAMPLES: usize = 10000;

#[test]
fn sampling_equals_plain_object() {
    let ref_sphere = Sphere::new(1.0, Material::Blank);
    let sphere = Sphere::new(0.1, Material::Blank)
        .rotate_x(crate::PI)
        .scale_uniform(10.0)
        .rotate_y(crate::PI)
        .rotate_z(crate::PI)
        .translate(1.0, 1.0, 1.0)
        .translate(-1.0, -1.0, -1.0);

    let xo = 2.0 * Point::NEG_Z;

    for _ in 0..NUM_SAMPLES {
        let wi = sphere.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let Some(hi) = sphere.hit(&ri, 0.0, crate::INF) else { panic!() };
        let Some(hi_ref) = ref_sphere.hit(&ri, 0.0, crate::INF) else { panic!() };
        let ref_p = ref_sphere.sample_towards_pdf(&ri, hi_ref.p, hi_ref.ng);
        let p = sphere.sample_towards_pdf(&ri, hi.p, hi.ng);

        assert!((ref_p - p).abs() < 1e-5);
    }
}

#[test]
fn area_equals_plain_object() {
    let ref_sphere = Sphere::new(1.0, Material::Blank);
    let sphere = Sphere::new(0.1, Material::Blank)
        .rotate_x(crate::PI)
        .scale_uniform(10.0)
        .rotate_y(crate::PI)
        .rotate_z(crate::PI);
    assert!((sphere.area() - ref_sphere.area()).abs() < crate::EPSILON);
}

#[test]
fn sampled_direction_hits() {
    let sphere = Sphere::new(0.1, Material::Blank)
        .translate(0.123, 0.456, 0.789)
        .scale_uniform(2.0)
        .rotate_x(crate::PI);

    let xo = Point::NEG_Z;

    for _ in 0..NUM_SAMPLES {
        let wi = sphere.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let Some(hi) = sphere.hit(&ri, 0.0, crate::INF) else { panic!() };
        let p = sphere.sample_towards_pdf(&ri, hi.p, hi.ng);

        assert!(p > 0.0);
    }
}
